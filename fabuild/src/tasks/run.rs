use std::{path::PathBuf, process::Command};

use crate::{
    ProgramEmptySubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    util::{generate_classpath, generate_launch_config, generate_log4j_config},
};

pub fn run(_args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    let root = PathBuf::from("../example");

    let project = load_root_project(&root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry();
    let tree = load_project_tree(&root, &project, &registry)?;

    generate_classpath(&root, &jregistry, &tree)?;
    generate_log4j_config(&root)?;
    generate_launch_config(&root, &jregistry, &project)?;

    Command::new("java")
        .arg("-cp")
        .arg(format!(
            "@{}",
            root.join("target")
                .join("classpath.tmp")
                .canonicalize()?
                .display()
        ))
        .arg(format!(
            "-Dfabric.classPathGroups={}",
            root.join("target")
                .join("classes")
                .canonicalize()?
                .display()
        ))
        .arg(format!(
            "-Dfabric.dli.config={}/launch.cfg",
            root.join("target").canonicalize()?.display()
        ))
        .arg("-Dfabric.dli.env=client")
        .arg("-Dfabric.dli.main=net.fabricmc.loader.impl.launch.knot.KnotClient")
        // TODO: remove these, maybe?
        .arg("--sun-misc-unsafe-memory-access=allow")
        .arg("--enable-native-access=ALL-UNNAMED")
        .arg("-Dfile.encoding=UTF-8")
        .arg("-Duser.country=US")
        .arg("-Duser.language=en")
        .arg("net.fabricmc.devlaunchinjector.Main")
        .current_dir(root.join("run"))
        .spawn()?
        .wait()?;

    Ok(())
}
