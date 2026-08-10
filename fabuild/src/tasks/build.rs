use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

use anyhow::Context;
use bhc_diagnostics::SourceMap;
use richrs::prelude::*;

use crate::{
    ProgramEmptySubCommand,
    error_parser::{self, JavaDiagnostic},
    error_styler::print_pretty_error,
    jregistry::load_default_jregistry,
    preprocessing::{resources::preprocess_resource_file, sources::preprocess_source_file},
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    tweaker::{get_class_tweakers, invoke_class_tweakers},
    util::{
        generate_client_classpath, generate_common_classpath, get_target_classes_folder,
        get_target_classtweakers_folder, get_target_client_classpath_file,
        get_target_common_classpath_file, get_target_sources_file,
    },
};

#[allow(clippy::too_many_lines)]
pub fn run(args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
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

    let root = &args.root;

    let project = load_root_project(root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry()?;
    let tree = load_project_tree(root, &project, &registry)?;
    generate_common_classpath(root, &jregistry, &tree)?;
    generate_client_classpath(root, &jregistry, &tree)?;

    // if the classtweakers changed, reinvoke quick-tweak
    for (new_path, new_filename) in get_class_tweakers(root)? {
        let old_path = get_target_classtweakers_folder(root).join(new_filename);

        if std::fs::exists(&old_path)? {
            if !std::fs::read(&old_path)
                .context(format!("reading from {}", old_path.display()))?
                .eq(&std::fs::read(&new_path)
                    .context(format!("reading from {}", new_path.display()))?)
            {
                invoke_class_tweakers(root, &project, &jregistry)?;
            }
        } else {
            invoke_class_tweakers(root, &project, &jregistry)?;
        }
    }

    let mut sources = vec![];
    iter_folder(&mut sources, &root.join("src/main/java"))?;
    iter_folder(&mut sources, &root.join("src/client/java"))?;

    // preprocess the sources first
    let target_sources = get_target_sources_file(root);
    let target_classes = get_target_classes_folder(root);

    // clean slate
    let _ = std::fs::remove_dir_all(&target_sources);
    let _ = std::fs::remove_dir_all(&target_classes);

    std::fs::create_dir_all(&target_sources)?;
    std::fs::create_dir_all(&target_classes)?;

    let mut processed_sources = HashMap::new();

    for file in &sources {
        let target = target_sources.join(PathBuf::from(
            &file
                .canonicalize()?
                .display()
                .to_string()
                .trim_start_matches(&root.join("src").canonicalize()?.display().to_string())
                .trim_start_matches('/')
                .trim_start_matches('\\'),
        ));
        std::fs::create_dir_all(target.parent().ok_or_else(|| {
            anyhow::anyhow!("failed to get parent of path {}", target.display())
        })?)?;

        let output = preprocess_source_file(&tree, file)?;
        std::fs::write(&target, output.clone())?;
        processed_sources.insert(target, output);
    }

    sources = vec![];
    iter_folder(&mut sources, &target_sources.join("main"))?;

    let mut client_sources = sources.clone();
    iter_folder(&mut client_sources, &target_sources.join("client"))?;

    // TODO: compare the current javac and the javac from JAVA_HOME
    let command_args = vec![
        "-Xlint:all".to_owned(),
        "-d".to_owned(),
        target_classes.canonicalize()?.display().to_string(),
        "-cp".to_owned(),
    ];

    let mut processes = vec![];
    let mut progress = Progress::new();

    let (mut shared_read, mut shared_write) = std::io::pipe()?;
    let (mut client_read, mut client_write) = std::io::pipe()?;

    let mut source_map = SourceMap::new();
    let mut file_ids = HashMap::new();

    for file in &client_sources {
        file_ids.insert(
            file.canonicalize()?,
            source_map.add_file(
                file.file_name()
                    .ok_or_else(|| anyhow::anyhow!("no filename for \"{}\"?", file.display()))?
                    .to_string_lossy()
                    .to_string(),
                processed_sources
                    .get(file)
                    .ok_or_else(|| {
                        anyhow::anyhow!("failed to read post-processed file: {}", file.display())
                    })?
                    .clone(),
            ),
        );
    }

    for (i, tasks) in [sources, client_sources].iter().enumerate() {
        processes.push((
            Command::new("javac")
                .args(
                    tasks
                        .iter()
                        .map(|t| Ok(t.canonicalize()?))
                        .collect::<anyhow::Result<Vec<PathBuf>>>()?,
                )
                .args(command_args.clone())
                .arg(format!(
                    "@{}",
                    if i == 0 {
                        get_target_common_classpath_file(root)
                    } else {
                        get_target_client_classpath_file(root)
                    }
                    .canonicalize()?
                    .display()
                ))
                .current_dir(&target_sources)
                .stdout(if i == 0 {
                    shared_write.try_clone()?
                } else {
                    client_write.try_clone()?
                })
                .stderr(if i == 0 {
                    shared_write.try_clone()?
                } else {
                    client_write.try_clone()?
                })
                .spawn()?,
            progress.add_task(
                if i == 0 { "Shared" } else { "Client" },
                Some(tasks.len() as u64),
                true,
            ),
        ));
    }

    let output = progress.render(40);
    print!("{}", output.to_ansi());
    let _ = std::io::stdout().flush();

    let thread_count = processes.len();
    let mut ok = true;

    for (thread, task) in &mut processes {
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
        print!("{}\x1b[0K", output.to_ansi());
        let _ = std::io::stdout().flush();
    }

    shared_write.flush()?;
    drop(shared_write);
    client_write.flush()?;
    drop(client_write);

    let mut shared_log = vec![];
    shared_read.read_to_end(&mut shared_log)?;
    let mut client_log = vec![];
    client_read.read_to_end(&mut client_log)?;

    let shared_log = shared_log
        .iter()
        .map(|s| (*s as char).to_string())
        .collect::<String>();
    let client_log = client_log
        .iter()
        .map(|s| (*s as char).to_string())
        .collect::<String>();

    let shared_parsed = error_parser::parse(&shared_log)?;
    let client_parsed = error_parser::parse(&client_log)?
        .iter()
        .filter(|s| !shared_parsed.contains(*s))
        .cloned()
        .collect::<Vec<JavaDiagnostic>>();

    print_pretty_error(&source_map, &file_ids, &shared_parsed)?;
    print_pretty_error(&source_map, &file_ids, &client_parsed)?;

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
                let path = jregistry.resolve_file(root, &name, &dep.version, jar)?;
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
    let mut raw_resources = vec![];
    iter_folder(&mut raw_resources, &root.join("src/main/resources"))?;
    iter_folder(&mut raw_resources, &root.join("src/client/resources"))?;

    let mut datagen_resources = vec![];
    iter_folder(&mut datagen_resources, &root.join("src/main/generated/"))?;

    for (_, task) in processes {
        progress.remove_task(task);
    }

    let mut resources = HashMap::new();

    for resource in &raw_resources {
        let out = preprocess_resource_file(&tree, resource)?;
        resources.insert(resource.clone(), out);
    }

    for resource in &datagen_resources {
        resources.insert(resource.clone(), std::fs::read(resource)?);
    }

    let task = progress.add_task(
        "Processing resources...",
        Some(resources.len() as u64),
        true,
    );

    println!();

    for (path, content) in &resources {
        // I'm incredibly sorry
        let binding = path.canonicalize()?.display().to_string();
        let relative = binding
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
            .trim_start_matches(
                &root
                    .join("src")
                    .join("main")
                    .join("generated")
                    .canonicalize()?
                    .display()
                    .to_string(),
            )
            .trim_start_matches('/')
            .trim_start_matches('\\');

        if !relative.starts_with(".cache") {
            let to = target_classes.join(relative);
            std::fs::create_dir_all(to.parent().ok_or_else(|| {
                anyhow::anyhow!("failed to find parent folder of {}", to.display())
            })?)?;
            std::fs::write(to, content)?;
        }

        progress.advance(task, 1)?;
        let output = progress.render(40);
        print!("\x1b[1A");
        print!("{}", output.to_ansi());
        let _ = std::io::stdout().flush();
    }

    Ok(())
}
