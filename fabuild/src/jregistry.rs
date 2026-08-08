use std::path::PathBuf;

/// The `JRegistry` contains references to the jar files.
pub struct FabuildJRegistry {
    pub root: PathBuf,
}

impl FabuildJRegistry {
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn resolve_path(&self, name: &str, version: &str) -> anyhow::Result<PathBuf> {
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

    pub fn resolve_file(
        &self,
        name: &str,
        version: &str,
        filename: &str,
    ) -> anyhow::Result<PathBuf> {
        Ok(self.resolve_path(name, version)?.join(filename))
    }
}

#[must_use]
pub fn load_default_jregistry() -> FabuildJRegistry {
    FabuildJRegistry::new("G:/steve/.fabuild".into())
}
