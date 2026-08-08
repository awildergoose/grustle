use std::path::PathBuf;

use crate::{
    ProgramEmptySubCommand, jregistry::load_default_jregistry, project::load_root_project,
    tweaker::invoke_class_tweakers,
};

pub fn run(_: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    let root = PathBuf::from("../example");

    let project = load_root_project(&root)?;
    let jregistry = load_default_jregistry();

    invoke_class_tweakers(&root, &project, &jregistry)?;

    Ok(())
}
