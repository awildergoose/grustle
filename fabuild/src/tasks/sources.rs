use std::process::{Command, Stdio};

use anyhow::Context;

use crate::{
    ProgramSourcesSubCommand,
    jregistry::load_default_jregistry,
    project::load_root_project,
    util::{get_target_aw_folder, get_target_vineflower_file},
};

pub fn run(args: &ProgramSourcesSubCommand) -> anyhow::Result<()> {
    let root = &args.root;

    let project = load_root_project(root)?;
    let jregistry = load_default_jregistry();

    let game_version = &project
        .dependencies
        .get("minecraft")
        .ok_or_else(|| anyhow::anyhow!("minecraft is not in the dependency list!"))?
        .version
        .clone();
    let game_client_version = &project
        .dependencies
        .get("minecraft-client")
        .ok_or_else(|| anyhow::anyhow!("minecraft-client is not in the dependency list!"))?
        .version
        .clone();

    let target_game = get_target_aw_folder(root);
    std::fs::create_dir_all(&target_game)?;

    let vineflower = &get_target_vineflower_file(root);
    std::fs::write(vineflower, include_bytes!("../../tools/vineflower.jar"))?;

    let vineflower = &vineflower.canonicalize().context(format!(
        "failed to canonicalize file path for vineflower.jar: {}",
        vineflower.display()
    ))?;

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(vineflower)
            .arg("--thread-count")
            .arg(args.threads.to_string())
            .arg(jregistry.resolve_file(root, "minecraft", game_version, "minecraft.jar")?) // input file
            .arg(target_game.join("minecraft-sources.jar")) // output file
            .stdout(Stdio::null())
            .spawn()?
            .wait()?
            .success(),
        "failed to generate sources for shared"
    );

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(vineflower)
            .arg("--thread-count")
            .arg(args.threads.to_string())
            .arg(jregistry.resolve_file(
                root,
                "minecraft-client",
                game_client_version,
                "minecraft-client.jar"
            )?) // input file
            .arg(target_game.join("minecraft-client-sources.jar")) // output file
            .stdout(Stdio::null())
            .spawn()?
            .wait()?
            .success(),
        "failed to generate sources for client"
    );

    println!("Finished generating sources!");

    Ok(())
}
