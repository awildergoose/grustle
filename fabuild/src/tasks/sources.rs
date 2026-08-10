use std::process::Command;

use anyhow::Context;

use crate::{
    ProgramSourcesSubCommand,
    jregistry::load_default_jregistry,
    project::load_root_project,
    util::{get_target_aw_folder, get_target_cachyflower_file},
};

pub fn run(args: &ProgramSourcesSubCommand) -> anyhow::Result<()> {
    let root = &args.root;

    let project = load_root_project(root)?;
    let jregistry = load_default_jregistry()?;

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

    let cachyflower = &get_target_cachyflower_file(root);
    std::fs::write(cachyflower, include_bytes!("../../tools/cachyflower.jar"))?;

    let cachyflower = &cachyflower.canonicalize().context(format!(
        "failed to canonicalize file path for cachyflower.jar: {}",
        cachyflower.display()
    ))?;

    let cf_cache = jregistry.resolve_cachyflower_cache();
    std::fs::create_dir_all(&cf_cache)?;
    let cf_cache = cf_cache.canonicalize()?;

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(cachyflower)
            .arg(args.threads.to_string())
            .arg(&cf_cache)
            .arg(jregistry.resolve_file(root, "minecraft", game_version, "minecraft.jar")?) // input file
            .arg(target_game.join("minecraft-sources.jar")) // output file
            .spawn()?
            .wait()?
            .success(),
        "failed to generate sources for shared"
    );

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(cachyflower)
            .arg(args.threads.to_string())
            .arg(&cf_cache)
            .arg(jregistry.resolve_file(
                root,
                "minecraft-client",
                game_client_version,
                "minecraft-client.jar"
            )?) // input file
            .arg(target_game.join("minecraft-client-sources.jar")) // output file
            .spawn()?
            .wait()?
            .success(),
        "failed to generate sources for client"
    );

    println!("Finished generating sources!");

    Ok(())
}
