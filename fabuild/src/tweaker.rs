use std::{path::Path, process::Command};

use crate::{
    jregistry::FabuildJRegistry,
    project::FabuildProject,
    util::{get_target_aw_folder, get_target_classtweakers_folder, get_target_qt_file},
};

pub fn invoke_class_tweakers(
    root: &Path,
    project: &FabuildProject,
    jregistry: &FabuildJRegistry,
) -> anyhow::Result<()> {
    let game_version = project
        .dependencies
        .get("minecraft")
        .ok_or_else(|| anyhow::anyhow!("minecraft is not in the dependency list!"))?;
    let game_client_version = project
        .dependencies
        .get("minecraft-client")
        .ok_or_else(|| anyhow::anyhow!("minecraft-client is not in the dependency list!"))?;

    let target_classtweakers = get_target_classtweakers_folder(root);
    let target_game = get_target_aw_folder(root);

    std::fs::create_dir_all(&target_classtweakers)?;
    std::fs::create_dir_all(&target_game)?;
    std::fs::write(get_target_qt_file(root), include_bytes!("../tools/qt.jar"))?;

    for ct in std::fs::read_dir(root.join("src").join("main").join("resources"))? {
        let ct = ct?;

        if ct.file_type()?.is_file() && ct.file_name().to_string_lossy().ends_with(".classtweaker")
        {
            std::fs::copy(ct.path(), target_classtweakers.join(ct.file_name()))?;
        }
    }

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(get_target_qt_file(root))
            .arg(jregistry.resolve_file("minecraft", game_version, "minecraft.jar")?) // minecraft shared jar
            .arg(jregistry.resolve_file("minecraft", game_version, "mappings.tiny")?) // mappings
            .arg(&target_classtweakers) // class tweakers folder
            .arg(target_game.join("minecraft.jar")) // final jar
            .spawn()?
            .wait()?
            .success(),
        "failed to run classtweakers for minecraft"
    );

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(get_target_qt_file(root))
            .arg(jregistry.resolve_file(
                "minecraft-client",
                game_client_version,
                "minecraft-client.jar",
            )?) // minecraft client jar
            .arg(jregistry.resolve_file("minecraft", game_version, "mappings.tiny")?) // mappings
            .arg(&target_classtweakers) // class tweakers folder
            .arg(target_game.join("minecraft-client.jar")) // final jar
            .spawn()?
            .wait()?
            .success(),
        "failed to run classtweakers for minecraft-client"
    );

    Ok(())
}
