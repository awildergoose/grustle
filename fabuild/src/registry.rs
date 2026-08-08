use std::path::PathBuf;

use anyhow::Context;

/// The Registry contains references to the package metadata files.
pub struct FabuildRegistry {
    pub root: PathBuf,
}

impl FabuildRegistry {
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn resolve_path(&self, name: &str, version: &str) -> anyhow::Result<PathBuf> {
        let split = name
            .split('.')
            .map(std::borrow::ToOwned::to_owned)
            .collect::<Vec<String>>();
        if split.len() == 1 {
            return Ok(self.root.join(name).join(version));
        }
        let resolved = split[0..split.len() - 1].join(".")
            + "/"
            + split
                .last()
                .ok_or_else(|| anyhow::anyhow!("failed to get artifact name"))?;
        let path = self.root.join(resolved).join(version);

        Ok(path)
    }

    pub fn resolve_package(&self, name: &str, version: &str) -> anyhow::Result<String> {
        let path = self.resolve_path(name, version)?.join("fabuild.toml");
        std::fs::read_to_string(&path)
            .context(format!("resolving package {name} to {}", path.display()))
    }
}

#[must_use]
pub fn load_default_registry() -> FabuildRegistry {
    FabuildRegistry::new("../registry/".into())
}
