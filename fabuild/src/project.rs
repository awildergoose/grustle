use std::{collections::HashMap, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{jregistry::FabuildJRegistry, registry::FabuildRegistry, util::SystemArchitecture};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildPackage {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildJava {
    pub version: String,
}

impl Default for FabuildJava {
    fn default() -> Self {
        // TODO
        Self {
            version: String::from("25"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FabuildPackageVersionNatives {
    pub x86: Vec<String>,
    pub x64: Vec<String>,
    pub arm64: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FabuildPackageVersion {
    pub runtime: Vec<String>,
    pub sources: Vec<String>,
    #[serde(default)]
    pub natives: FabuildPackageVersionNatives,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildProject {
    pub package: FabuildPackage,
    #[serde(default)]
    pub java: FabuildJava,
    pub dependencies: HashMap<String, String>,
    pub versions: HashMap<String, FabuildPackageVersion>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildResolvedProject {
    pub package: FabuildPackage,
    #[serde(default)]
    pub java: FabuildJava,
    pub dependencies: HashMap<String, String>,
    pub version: FabuildPackageVersion,
    pub current_version: String,
}

impl FabuildResolvedProject {
    #[must_use]
    pub fn get_full_name(&self) -> String {
        if self.package.path.is_empty() {
            return self.package.name.clone();
        }
        format!("{}.{}", self.package.path, self.package.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildProjectTree {
    pub root: FabuildProject,
    pub packages: Vec<FabuildResolvedProject>,
}

impl FabuildProjectTree {
    pub fn gather_classpath(
        &self,
        root: &Path,
        jregistry: &FabuildJRegistry,
        sources: bool,
        architecture: SystemArchitecture,
    ) -> anyhow::Result<Vec<String>> {
        anyhow::ensure!(
            architecture != SystemArchitecture::Auto,
            "resolve the architecture before passing it onto any functions"
        );

        let mut out: Vec<String> = Vec::new();

        // resolve dependencies first
        for package in &self.packages {
            if sources {
                for filename in &package.version.sources {
                    let resolved = jregistry.resolve_jar(
                        &package.get_full_name(),
                        &package.current_version,
                        filename,
                    )?;
                    out.push(resolved.display().to_string());
                }
            } else {
                for filename in &package.version.runtime {
                    let resolved = jregistry.resolve_jar(
                        &package.get_full_name(),
                        &package.current_version,
                        filename,
                    )?;
                    out.push(resolved.display().to_string());
                }

                for filename in match architecture {
                    SystemArchitecture::X86 => &package.version.natives.x86,
                    SystemArchitecture::X64 => &package.version.natives.x64,
                    SystemArchitecture::Arm64 => &package.version.natives.arm64,
                    SystemArchitecture::Auto => unreachable!(),
                } {
                    let resolved = jregistry.resolve_jar(
                        &package.get_full_name(),
                        &package.current_version,
                        filename,
                    )?;
                    out.push(resolved.display().to_string());
                }
            }
        }

        // now, resolve the main project
        out.push(format!(
            "{}/target/class",
            std::fs::canonicalize(root)?.display()
        ));

        Ok(out)
    }
}

pub fn parse_project(text: &str) -> anyhow::Result<FabuildProject> {
    toml::from_str(text)
        .map_err(|e| anyhow::anyhow!(format!("Failed to parse Fabuild project: {e:#?}")))
}

pub fn load_root_project(root: &Path) -> anyhow::Result<FabuildProject> {
    // if std::fs::exists("fabuild.toml").is_ok_and(|s| s) {
    //     return parse_project(&std::fs::read_to_string("fabuild.toml")?);
    // }

    // anyhow::bail!("failed to load project")
    parse_project(
        &std::fs::read_to_string(root.join("fabuild.toml")).context("loading the root project")?,
    )
}

/// Loads the project and its dependencies into a `FabuildProjectTree`.
/// This is expected to be called on the *root* project only.
pub fn load_project_tree(
    root: &Path,
    project: FabuildProject,
    registry: &FabuildRegistry,
) -> anyhow::Result<FabuildProjectTree> {
    fn resolve_dependencies(
        packages: &mut Vec<FabuildResolvedProject>,
        package: &FabuildProject,
        registry: &FabuildRegistry,
    ) -> anyhow::Result<()> {
        let parent_name = format!("{}.{}", package.package.path, package.package.name);

        if packages.iter().any(|p| p.get_full_name().eq(&parent_name)) {
            return Ok(());
        }

        for (dependency, version) in &package.dependencies {
            if *dependency == parent_name {
                println!("circular dependency found! {dependency}");
                break;
            }

            if packages.iter().any(|p| p.get_full_name().eq(dependency)) {
                continue;
            }

            let p = parse_project(&registry.resolve_package(dependency)?)
                .context(format!("loading {dependency}"))?;
            resolve_dependencies(packages, &p, registry)?;
            let name = format!("{}.{}", p.package.path, p.package.name);
            let resolved = FabuildResolvedProject {
                package: p.package,
                java: p.java,
                dependencies: p.dependencies,
                version: p
                    .versions
                    .get(version)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "failed to get version {version} for {name} (required by {parent_name})",
                        )
                    })?
                    .clone(),
                current_version: version.clone()
            };
            packages.push(resolved);
        }

        Ok(())
    }

    // TODO: store hash in the lock
    if std::fs::exists(root.join("fabuild.lock")).is_ok_and(|s| s) {
        return Ok(toml::from_str(&std::fs::read_to_string(
            root.join("fabuild.lock"),
        )?)?);
    }

    let mut packages = vec![];

    resolve_dependencies(&mut packages, &project, registry)?;

    let tree = FabuildProjectTree {
        root: project,
        packages,
    };

    std::fs::write(root.join("fabuild.lock"), toml::to_string(&tree)?)?;

    Ok(tree)
}
