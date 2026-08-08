use std::{path::Path, process::Command};

use crate::{jregistry::FabuildJRegistry, project::FabuildProject};

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

    std::fs::create_dir_all(root.join("target").join("classtweakers"))?;
    std::fs::create_dir_all(root.join("target").join("game"))?;
    std::fs::write(
        root.join("target").join("qt.jar"),
        include_bytes!("../tools/qt.jar"),
    )?;

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(root.join("target").join("qt.jar"))
            .arg(jregistry.resolve_file("minecraft", game_version, "minecraft.jar")?) // minecraft shared jar
            .arg(jregistry.resolve_file("minecraft", game_version, "mappings.tiny")?) // mappings
            .arg(root.join("target").join("classtweakers")) // class tweakers folder
            .arg(root.join("target").join("game").join("minecraft.jar")) // final jar
            .spawn()?
            .wait()?
            .success(),
        "failed to run classtweakers for minecraft"
    );

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(root.join("target").join("qt.jar"))
            .arg(jregistry.resolve_file(
                "minecraft-client",
                game_client_version,
                "minecraft-client.jar",
            )?) // minecraft client jar
            .arg(jregistry.resolve_file("minecraft", game_version, "mappings.tiny")?) // mappings
            .arg(root.join("target").join("classtweakers")) // class tweakers folder
            .arg(
                root.join("target")
                    .join("game")
                    .join("minecraft-client.jar"),
            ) // final jar
            .spawn()?
            .wait()?
            .success(),
        "failed to run classtweakers for minecraft-client"
    );

    Ok(())
}
