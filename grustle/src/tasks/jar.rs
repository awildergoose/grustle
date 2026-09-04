use std::process::Command;

use anyhow::Context;

use crate::{
    commands::{ProfilefulArg, ProgramJarSubCommand},
    project::load_root_project,
    util::{get_target_classes_folder, get_target_folder},
};

pub fn run(args: &ProgramJarSubCommand) -> anyhow::Result<()> {
    let root = &args.root;
    let profile = args.profile();

    let project = load_root_project(root)?;
    let version = &project.version;

    // TODO: check if access wideners were even created

    let classes = get_target_classes_folder(root, profile);
    std::fs::create_dir_all(&classes)?;

    anyhow::ensure!(
        Command::new("jar")
            .arg("cvf")
            .arg(get_target_folder(root, profile).canonicalize()?.join(
                version.runtime.first().ok_or_else(|| {
                    anyhow::anyhow!(
                        "no jar runtime filenames found in root prooject ({})",
                        project.get_full_name()
                    )
                })?
            ),)
            .arg(".")
            .current_dir(classes)
            .spawn()
            .context("spawning jar utility")?
            .wait()?
            .success(),
        "failed to package files into a jar"
    );

    Ok(())
}
