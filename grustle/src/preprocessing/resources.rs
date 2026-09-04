use std::path::Path;

use anyhow::Context;

use crate::project::GrustleProjectTree;

pub fn preprocess_resource_file(tree: &GrustleProjectTree, path: &Path) -> anyhow::Result<Vec<u8>> {
    let ext = path
        .extension()
        .ok_or_else(|| anyhow::anyhow!("resource doesn't have a file extension"))?
        .to_string_lossy()
        .to_string();
    let content = std::fs::read(path)?;

    if !["json", "jsonc", "toml"]
        .iter()
        .map(std::string::ToString::to_string)
        .any(|x| x == ext)
    {
        return Ok(content);
    }

    let mut out = String::try_from(content).context(format!(
        "file {} doesn't have valid UTF-8 (preprocessing)",
        path.display()
    ))?;

    out = out.replace("${project.name}", &tree.root.package.name);
    out = out.replace("${project.path}", &tree.root.package.path);
    out = out.replace("${project.version}", &tree.root.package.ver);

    for (key, value) in &tree.root.extra {
        out = out.replace(&format!("${{project.extra.{key}}}"), value);
    }

    for package in &tree.packages {
        out = out.replace(
            &format!("${{project.packages.{}.version}}", package.get_full_name()),
            &package.ver,
        );
    }

    Ok(out.into())
}
