use std::{io::Write, path::PathBuf, process::Command};

use anyhow::Context;
use richrs::prelude::*;

use crate::{
    ProgramBuildSubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    util::generate_classpath,
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

    let mut sources = vec![];
    iter_folder(&mut sources, &root.join("src/main/java"))?;
    iter_folder(&mut sources, &root.join("src/client/java"))?;

    // TODO: compare the current javac and the javac from JAVA_HOME
    let command_args = vec![
        "-d".to_owned(),
        root.join("target").join("classes").display().to_string(),
        "-cp".to_owned(),
        format!("@{}", root.join("target").join("classpath.tmp").display()),
    ];

    let mut threads = vec![];
    let mut progress = Progress::new();

    for tasks in sources.chunks(args.jobs) {
        threads.push((
            Command::new("javac")
                .args(tasks)
                .args(command_args.clone())
                .current_dir(&root)
                .spawn()?,
            progress.add_task(
                tasks
                    .iter()
                    .map(|s| {
                        Ok::<String, anyhow::Error>(
                            s.file_name()
                                .ok_or_else(|| anyhow::anyhow!("non UTF-8 filename!"))?
                                .to_string_lossy()
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
    print!("\x1b[{}A", 3);
    print!("{}", output.to_ansi());
    let _ = std::io::stdout().flush();

    for (thread, task) in &mut threads {
        thread.wait()?;

        progress.advance(
            *task,
            progress
                .get_task(*task)
                .ok_or_else(|| unreachable!())?
                .total
                .ok_or_else(|| unreachable!())?,
        )?;
        let output = progress.render(40);
        print!("\x1b[{}A", 3);
        print!("{}", output.to_ansi());
        let _ = std::io::stdout().flush();
    }

    // copy resources for now, later on, we can pre-process them!
    let mut resources = vec![];
    iter_folder(&mut resources, &root.join("src/main/resources"))?;
    iter_folder(&mut resources, &root.join("src/client/resources"))?;

    for resource in &resources {
        let from = resource;
        let to = root.join("target").join("classes").join(
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
    }

    Ok(())
}
