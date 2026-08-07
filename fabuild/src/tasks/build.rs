use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    ProgramEmptySubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    util::SystemArchitecture,
};

pub fn run(_args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    fn iter_folder(sources: &mut Vec<PathBuf>, path: &PathBuf) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                iter_folder(sources, &entry.path())?;
            } else if file_type.is_file() {
                if Path::new(&entry.file_name())
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("java"))
                {
                    sources.push(entry.path());
                }
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
    let tree = load_project_tree(&root, project, &registry)?;
    let entries = tree.gather_classpath(
        &root,
        &jregistry,
        SystemArchitecture::Auto.resolve(),
        true,
        true,
    )?;
    let classes: Vec<String> = entries.iter().flat_map(|s| s.classes.clone()).collect();
    std::fs::write(root.join("target").join("classpath.tmp"), classes.join(";"))?;

    let mut sources = vec![];
    iter_folder(&mut sources, &root.join("src"))?;

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
        .current_dir(root)
        .spawn()?
        .wait()?;

    Ok(())
}
