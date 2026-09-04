use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    jregistry::GrustleJRegistry,
    project::GrustleProject,
    util::{get_target_aw_folder, get_target_classtweakers_folder, get_target_qt_file},
};

pub fn get_class_tweakers(root: &Path) -> anyhow::Result<Vec<(PathBuf, String)>> {
    let mut list = vec![];

    for ct in std::fs::read_dir(root.join("src").join("main").join("resources"))? {
        let ct = ct?;

        if ct.file_type()?.is_file() && ct.file_name().to_string_lossy().ends_with(".classtweaker")
        {
            list.push((ct.path(), ct.file_name().to_string_lossy().to_string()));
        }
    }

    Ok(list)
}

pub fn invoke_class_tweakers(
    root: &Path,
    profile: &str,
    project: &GrustleProject,
    jregistry: &GrustleJRegistry,
) -> anyhow::Result<()> {
    let game_version = project
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

    let target_classtweakers = get_target_classtweakers_folder(root, profile);
    let target_game = get_target_aw_folder(root, profile);

    std::fs::create_dir_all(&target_classtweakers)?;
    std::fs::create_dir_all(&target_game)?;
    std::fs::write(
        get_target_qt_file(root, profile),
        include_bytes!("../tools/qt.jar"),
    )?;

    for (path, file_name) in get_class_tweakers(root)? {
        std::fs::copy(path, target_classtweakers.join(file_name))?;
    }

    anyhow::ensure!(
        Command::new("java")
            .arg("-jar")
            .arg(get_target_qt_file(root, profile))
            .arg(
                jregistry
                    .resolve_path("minecraft", &game_version)?
                    .join("minecraft.jar")
            ) // minecraft shared jar
            .arg(
                jregistry
                    .resolve_path("minecraft", &game_version)?
                    .join("mappings.tiny")
            ) // mappings
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
            .arg(get_target_qt_file(root, profile))
            .arg(
                jregistry
                    .resolve_path("minecraft-client", game_client_version)?
                    .join("minecraft-client.jar")
            ) // minecraft client jar
            .arg(
                jregistry
                    .resolve_path("minecraft", &game_version)?
                    .join("mappings.tiny")
            ) // mappings
            .arg(&target_classtweakers) // class tweakers folder
            .arg(target_game.join("minecraft-client.jar")) // final jar
            .spawn()?
            .wait()?
            .success(),
        "failed to run classtweakers for minecraft-client"
    );

    Ok(())
}
