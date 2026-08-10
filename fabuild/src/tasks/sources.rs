use std::process::{Command, Stdio};

use crate::{
    ProgramEmptySubCommand,
    jregistry::load_default_jregistry,
    project::load_root_project,
    util::{get_target_aw_folder, get_target_vineflower_file},
};

pub fn run(args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
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

    std::fs::write(
        get_target_vineflower_file(root),
        include_bytes!("../../tools/vineflower.jar"),
    )?;

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(get_target_vineflower_file(root))
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
            .arg(get_target_vineflower_file(root))
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
