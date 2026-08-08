use std::{io::Write, path::PathBuf, process::Command};

use anyhow::Context;
use richrs::prelude::*;

use crate::{
    ProgramBuildSubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    tweaker::{get_class_tweakers, invoke_class_tweakers},
    util::{
        generate_classpath, get_target_classes_folder, get_target_classpath_file,
        get_target_classtweakers_folder, split_jobs,
    },
};

#[allow(clippy::too_many_lines)]
pub fn run(args: &ProgramBuildSubCommand) -> anyhow::Result<()> {
    fn iter_folder(sources: &mut Vec<PathBuf>, path: &PathBuf) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                iter_folder(sources, &entry.path())?;
            } else if file_type.is_file() {
                sources.push(entry.path());
            } else {
                anyhow::bail!("unhandled file type: {file_type:#?}");
            }
        }

        Ok(())
    }

    let root = PathBuf::from("../example");

    let project = load_root_project(&root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry();
    let tree = load_project_tree(&root, &project, &registry)?;
    generate_classpath(&root, &jregistry, &tree)?;

    // if the classtweakers changed,
    for (new_path, new_filename) in get_class_tweakers(&root)? {
        let old_path = get_target_classtweakers_folder(&root).join(new_filename);

        if std::fs::exists(&old_path)? {
            if !std::fs::read(&old_path)
                .context(format!("reading from {}", old_path.display()))?
                .eq(&std::fs::read(&new_path)
                    .context(format!("reading from {}", new_path.display()))?)
            {
                invoke_class_tweakers(&root, &project, &jregistry)?;
            }
        } else {
            invoke_class_tweakers(&root, &project, &jregistry)?;
        }
    }

    let mut sources = vec![];
    iter_folder(&mut sources, &root.join("src/main/java"))?;
    iter_folder(&mut sources, &root.join("src/client/java"))?;

    // TODO: compare the current javac and the javac from JAVA_HOME
    let command_args = vec![
        "-d".to_owned(),
        get_target_classes_folder(&root).display().to_string(),
        "-cp".to_owned(),
        format!("@{}", get_target_classpath_file(&root).display()),
    ];

    let mut threads = vec![];
    let mut progress = Progress::new();

    for tasks in split_jobs(sources, args.jobs) {
        threads.push((
            Command::new("javac")
                .args(tasks.clone())
                .args(command_args.clone())
                .current_dir(&root)
                .spawn()?,
            progress.add_task(
                tasks
                    .iter()
                    .map(|s| {
                        let filename = s
                            .file_name()
                            .ok_or_else(|| anyhow::anyhow!("non UTF-8 filename!"))?
                            .to_string_lossy();

                        Ok::<String, anyhow::Error>(
                            filename
                                .split('.')
                                .next()
                                .unwrap_or_else(|| &filename)
                                .to_string(),
                        )
                    })
                    .collect::<anyhow::Result<Vec<String>>>()?
                    .join(", ")
                    .clone(),
                Some(tasks.len() as u64),
                true,
            ),
        ));
    }

    let output = progress.render(40);
    print!("{}", output.to_ansi());
    let _ = std::io::stdout().flush();

    let thread_count = threads.len();
    let mut ok = true;

    for (thread, task) in &mut threads {
        if !thread.wait()?.success() {
            ok = false;
        }

        progress.advance(
            *task,
            progress
                .get_task(*task)
                .ok_or_else(|| unreachable!())?
                .total
                .ok_or_else(|| unreachable!())?,
        )?;
        let output = progress.render(40);
        print!("\x1b[{thread_count}A");
        print!("{}", output.to_ansi());
        let _ = std::io::stdout().flush();
    }

    anyhow::ensure!(ok, "failed to compile java code");

    // copy resources for now, later on, we can pre-process them!
    let mut resources = vec![];
    iter_folder(&mut resources, &root.join("src/main/resources"))?;
    iter_folder(&mut resources, &root.join("src/client/resources"))?;

    for (_, task) in threads {
        progress.remove_task(task);
    }

    let task = progress.add_task("Copying resources...", Some(resources.len() as u64), true);

    println!();

    for resource in &resources {
        let from = resource;
        let to = get_target_classes_folder(&root).join(
            resource
                .canonicalize()?
                .display()
                .to_string()
                .trim_start_matches(
                    &root
                        .join("src")
                        .join("main")
                        .join("resources")
                        .canonicalize()?
                        .display()
                        .to_string(),
                )
                .trim_start_matches(
                    &root
                        .join("src")
                        .join("client")
                        .join("resources")
                        .canonicalize()?
                        .display()
                        .to_string(),
                )
                .trim_start_matches('/')
                .trim_start_matches('\\'),
        );
        std::fs::create_dir_all(
            to.parent().ok_or_else(|| {
                anyhow::anyhow!("failed to find parent folder of {}", to.display())
            })?,
        )?;
        std::fs::copy(from, &to).context(format!(
            "copying resource from {} to {}",
            from.display(),
            to.display()
        ))?;

        progress.advance(task, 1)?;
        let output = progress.render(40);
        print!("\x1b[1A");
        print!("{}", output.to_ansi());
        let _ = std::io::stdout().flush();
    }

    Ok(())
}
