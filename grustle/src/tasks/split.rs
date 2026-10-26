use crate::{
    commands::{ProfilefulArg, ProgramEmptySubCommand},
    jregistry::load_default_jregistry,
    project::load_root_project,
    splitter::invoke_splitter,
};

pub fn run(args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    let root = &args.root;
    let profile = args.profile();

    let project = load_root_project(root)?;
    let jregistry = load_default_jregistry()?;

    invoke_splitter(root, profile, &project, &jregistry)?;

    Ok(())
}
