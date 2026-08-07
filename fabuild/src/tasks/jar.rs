use std::{path::PathBuf, process::Command};

use crate::{ProgramEmptySubCommand, project::load_root_project};

pub fn run(_args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    let root = PathBuf::from("../example");

    let project = load_root_project(&root)?;
    let version = project
        .versions
        .get(&project.package.version)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "version \"{}\" not found in root project ({})",
                project.package.version,
                project.get_full_name()
            )
        })?;

    Command::new("jar")
        .arg("cvf")
        .arg(
            root.join("target")
                .canonicalize()?
                .join(version.runtime.first().ok_or_else(|| {
                    anyhow::anyhow!(
                        "no jar runtime filenames found in root prooject ({})",
                        project.get_full_name()
                    )
                })?),
        )
        .arg(".")
        .current_dir(root.join("target").join("classes"))
        .spawn()?
        .wait()?;

    Ok(())
}
