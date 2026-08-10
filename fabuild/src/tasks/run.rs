use anyhow::Context;
use std::process::Command;

use crate::{
    ProgramRunSubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    util::{
        generate_client_classpath, generate_launch_config, generate_log4j_config, get_run_folder,
        get_target_classes_folder, get_target_client_classpath_file, get_target_launch_folder,
    },
};

pub fn run(args: &ProgramRunSubCommand) -> anyhow::Result<()> {
    let root = &args.root;

    let project = load_root_project(root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry()?;
    let tree = load_project_tree(root, &project, &registry)?;

    let run_folder = get_run_folder(root);
    let client_classpath_file = get_target_client_classpath_file(root);
    let classes_folder = get_target_classes_folder(root);
    let launch_folder = get_target_launch_folder(root);

    std::fs::create_dir_all(&run_folder)?;
    std::fs::create_dir_all(&classes_folder)?;
    std::fs::create_dir_all(&launch_folder)?;

    generate_client_classpath(root, &jregistry, &tree)?;
    generate_log4j_config(root)?;
    generate_launch_config(root, &jregistry, &project)?;

    let mut binding = Command::new("java");
    let mut command = binding
        .current_dir(run_folder)
        .arg("-cp")
        .arg(format!(
            "@{}",
            client_classpath_file.canonicalize()?.display()
        ))
        .arg(format!(
            "-Dfabric.dli.config={}/launch.cfg",
            launch_folder.canonicalize()?.display()
        ))
        // TODO: remove these, maybe?
        .arg("--sun-misc-unsafe-memory-access=allow")
        .arg("--enable-native-access=ALL-UNNAMED")
        .arg("-Dfile.encoding=UTF-8")
        .arg("-Duser.country=US")
        .arg("-Duser.language=en");

    if args.server {
        command = command
            .arg("-Dfabric.dli.env=server")
            .arg("-Dfabric.dli.main=net.fabricmc.loader.impl.launch.knot.KnotServer");
    } else if args.datagen {
        let out = root.join("src").join("main").join("generated");
        std::fs::create_dir_all(&out)?;

        command = command
            .arg("-Dfabric-api.datagen")
            .arg(format!(
                "-Dfabric-api.datagen.output-dir={}",
                out.canonicalize()
                    .context("canonicalizing datagen output folder")?
                    .display()
            ))
            .arg("-Dfabric.dli.env=client")
            .arg("-Dfabric.dli.main=net.fabricmc.loader.impl.launch.knot.KnotClient");
    } else {
        command = command
            .arg(format!(
                "-Dfabric.classPathGroups={}",
                classes_folder.canonicalize()?.display()
            ))
            .arg("-Dfabric.dli.env=client")
            .arg("-Dfabric.dli.main=net.fabricmc.loader.impl.launch.knot.KnotClient");
    }

    command = command.arg("net.fabricmc.devlaunchinjector.Main");

    if args.server {
        command = command.arg("nogui");
    }

    anyhow::ensure!(command.spawn()?.wait()?.success(), "failed to run game");

    Ok(())
}
