use crate::{
    commands::{ProfilefulArg, ProgramClasspathArgs},
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
};

pub fn run(args: &ProgramClasspathArgs) -> anyhow::Result<()> {
    let root = &args.root;
    let profile = args.profile();

    let project = load_root_project(root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry()?;
    let tree = load_project_tree(root, &project, &registry)?;
    let entries =
        tree.gather_classpath(root, profile, &jregistry, args.arch.resolve(), true, true)?;

    let mut out = Vec::new();

    for entry in entries {
        let t = if args.sources {
            entry.sources
        } else {
            entry.classes
        };
        out.extend_from_slice(&t);
    }

    println!("{}", out.join(";"));

    Ok(())
}
