use std::{path::PathBuf, process::Command};

use anyhow::Context;

use crate::{
    ProgramEmptySubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    util::generate_classpath,
};

pub fn run(_args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
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
    Command::new("javac")
        .args(sources)
        .arg("-d")
        .arg(root.join("target").join("classes"))
        .arg("-cp")
        .arg(format!(
            "@{}",
            root.join("target").join("classpath.tmp").display()
        ))
        .current_dir(&root)
        .spawn()?
        .wait()?;

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
