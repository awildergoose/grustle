use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

use anyhow::Context;
use richrs::prelude::*;

use crate::{
    ProgramBuildSubCommand,
    jregistry::load_default_jregistry,
    preprocessing::sources::preprocess_source_file,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    tweaker::{get_class_tweakers, invoke_class_tweakers},
    util::{
        generate_classpath, get_target_classes_folder, get_target_classpath_file,
        get_target_classtweakers_folder, get_target_sources_file, split_jobs,
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

    // if the classtweakers changed, reinvoke quick-tweak
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

    // preprocess the sources first
    let target_sources = get_target_sources_file(&root);
    let target_classes = get_target_classes_folder(&root);

    std::fs::create_dir_all(&target_sources)?;
    std::fs::create_dir_all(&target_classes)?;

    for file in &sources {
        let target = target_sources.join(PathBuf::from(
            &file
                .canonicalize()?
                .display()
                .to_string()
                .trim_start_matches(&root.join("src").canonicalize()?.display().to_string())
                .trim_start_matches(&root.join("src").canonicalize()?.display().to_string())
                .trim_start_matches('/')
                .trim_start_matches('\\'),
        ));
        std::fs::create_dir_all(target.parent().ok_or_else(|| {
            anyhow::anyhow!("failed to get parent of path {}", target.display())
        })?)?;

        preprocess_source_file(&tree, file, &target)?;
    }

    sources = vec![];
    iter_folder(&mut sources, &target_sources)?;

    // TODO: compare the current javac and the javac from JAVA_HOME
    let command_args = vec![
        "-d".to_owned(),
        target_classes.canonicalize()?.display().to_string(),
        "-cp".to_owned(),
        format!(
            "@{}",
            get_target_classpath_file(&root).canonicalize()?.display()
        ),
    ];

    let mut threads = vec![];
    let mut progress = Progress::new();

    let (mut stdout_read, mut stdout_write) = std::io::pipe()?;

    for tasks in split_jobs(sources, args.jobs) {
        threads.push((
            Command::new("javac")
                .args(
                    tasks
                        .iter()
                        .map(|t| Ok(t.canonicalize()?))
                        .collect::<anyhow::Result<Vec<PathBuf>>>()?,
                )
                .args(command_args.clone())
                .current_dir(&target_sources)
                .stdout(stdout_write.try_clone()?)
                .stderr(stdout_write.try_clone()?)
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

    stdout_write.flush()?;
    drop(stdout_write);

    let mut log = vec![];
    stdout_read.read_to_end(&mut log)?;

    println!(
        "{}",
        log.iter()
            .map(|s| (*s as char).to_string())
            .collect::<String>()
    );

    anyhow::ensure!(ok, "failed to compile java code");

    // copy embedded jars' contents into there aswell!
    for (name, dep) in project.dependencies {
        if dep.embedded {
            let pkg = tree
                .packages
                .iter()
                .find(|p| p.get_full_name() == name)
                .ok_or_else(|| {
                    anyhow::anyhow!("package is not in the project tree? this should never happen")
                })?;

            for jar in &pkg.version.runtime {
                let path = jregistry.resolve_file(&root, &name, &dep.version, jar)?;
                let mut zip = zip::read::ZipArchive::new(
                    File::open(&path).context(format!("opening jar at {}", path.display()))?,
                )
                .context(format!("parsing jar at {}", path.display()))?;

                for index in 0..zip.len() {
                    let mut file = zip.by_index(index)?;
                    let filename = file.enclosed_name().context(format!(
                        "unsafe zip file entry was found in {}",
                        path.display()
                    ))?;

                    if filename.starts_with("META-INF") {
                        continue;
                    }

                    let filepath = target_classes.join(&dep.location).join(&filename);
                    let mut folder = filepath.clone();

                    if file.is_file() {
                        folder = folder
                            .parent()
                            .ok_or_else(|| {
                                anyhow::anyhow!("no parent found for {}", folder.display())
                            })?
                            .to_path_buf();
                    }

                    std::fs::create_dir_all(folder)?;

                    let mut contents = vec![];
                    let _ = file.read_to_end(&mut contents)?;
                    std::fs::write(filepath, contents)?;
                }
            }
        }
    }

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
        let to = target_classes.join(
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
