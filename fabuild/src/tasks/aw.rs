use crate::{
    ProgramEmptySubCommand, jregistry::load_default_jregistry, project::load_root_project,
    tweaker::invoke_class_tweakers,
};

pub fn run(args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    let root = &args.root;

    let project = load_root_project(root)?;
    let jregistry = load_default_jregistry();

    invoke_class_tweakers(root, &project, &jregistry)?;

    Ok(())
}
