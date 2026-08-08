use std::path::Path;

use crate::project::FabuildProject;

pub fn preprocess_source_file(
    project: &FabuildProject,
    path: &Path,
    out_path: &Path,
) -> anyhow::Result<()> {
    let filename = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("filename is non UTF-8"))?
        .to_string_lossy()
        .to_string();
    let mut content = std::fs::read_to_string(path)?;

    // #if version(minecraft) >= 26.0 {}
    for (line_index, line) in content
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>()
        .iter()
        .enumerate()
    {
        if line.starts_with("//#") {
            // assume this is a preprocessor directive
            let line = line.trim_start_matches("//#").to_string();
            let directive = line
                .split(' ')
                .next()
                .ok_or_else(|| anyhow::anyhow!("no directive was specified"))?;

            match directive {
                "if" => {}
                _ => {
                    anyhow::bail!("unknown directive: {directive} (from {filename}:L{line_index})");
                }
            }
        }
    }

    std::fs::write(out_path, content)?;

    Ok(())
}
