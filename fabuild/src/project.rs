use std::{collections::HashMap, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    jregistry::FabuildJRegistry,
    registry::FabuildRegistry,
    util::{SystemArchitecture, get_target_classes_folder},
};

// Spaghetti
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildPackage {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub ver: String,
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
pub struct FabuildDependency {
    pub version: String,
    #[serde(default)]
    pub embedded: bool,
    #[serde(default)]
    pub location: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum FabuildSeDependency {
    Versioned(String),
    Full(FabuildDependency),
}

impl FabuildSeDependency {
    #[must_use]
    pub fn resolve(&self) -> FabuildDependency {
        match self {
            Self::Versioned(v) => FabuildDependency {
                version: v.clone(),
                embedded: false,
                location: String::new(),
            },
            Self::Full(d) => d.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildSeProject {
    pub package: FabuildPackage,
    pub dependencies: HashMap<String, FabuildSeDependency>,
    pub version: FabuildPackageVersion,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildProject {
    pub package: FabuildPackage,
    pub dependencies: HashMap<String, FabuildDependency>,
    pub version: FabuildPackageVersion,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl FabuildProject {
    #[must_use]
    pub fn get_full_name(&self) -> String {
        if self.package.path.is_empty() {
            return self.package.name.clone();
        }

        format!("{}.{}", self.package.path, self.package.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabuildResolvedProject {
    pub package: FabuildPackage,
    pub dependencies: HashMap<String, FabuildDependency>,
    pub version: FabuildPackageVersion,
    pub ver: String,
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

pub struct ClasspathEntry {
    pub classes: Vec<String>,
    pub sources: Vec<String>,
}

impl FabuildProjectTree {
    /// Returns (Vec<Classpath>, Vec<Sourcepath>)
    pub fn gather_classpath(
        &self,
        root: &Path,
        jregistry: &FabuildJRegistry,
        architecture: SystemArchitecture,
        should_resolve_root: bool,
        include_natives: bool,
    ) -> anyhow::Result<Vec<ClasspathEntry>> {
        anyhow::ensure!(
            architecture != SystemArchitecture::Auto,
            "resolve the architecture before passing it onto any functions"
        );

        let mut out = Vec::new();

        // resolve dependencies first
        for package in &self.packages {
            let mut entry = ClasspathEntry {
                classes: vec![],
                sources: vec![],
            };

            for filename in &package.version.sources {
                let resolved = jregistry.resolve_file(
                    root,
                    &package.get_full_name(),
                    &package.ver,
                    filename,
                )?;
                entry.sources.push(resolved.display().to_string());
            }

            for filename in &package.version.runtime {
                let resolved = jregistry.resolve_file(
                    root,
                    &package.get_full_name(),
                    &package.ver,
                    filename,
                )?;
                entry.classes.push(resolved.display().to_string());
            }

            if include_natives {
                for filename in match architecture {
                    SystemArchitecture::X86 => &package.version.natives.x86,
                    SystemArchitecture::X64 => &package.version.natives.x64,
                    SystemArchitecture::Arm64 => &package.version.natives.arm64,
                    SystemArchitecture::Auto => unreachable!(),
                } {
                    let resolved = jregistry.resolve_file(
                        root,
                        &package.get_full_name(),
                        &package.ver,
                        filename,
                    )?;
                    entry.classes.push(resolved.display().to_string());
                }
            }

            out.push(entry);
        }

        // now, resolve the main project
        if should_resolve_root {
            out.push(ClasspathEntry {
                classes: vec![format!(
                    "{}",
                    get_target_classes_folder(root)
                        .canonicalize()
                        .context("classes folder hasn't been created yet")?
                        .display()
                )],
                sources: vec![],
            });
        }

        Ok(out)
    }
}

pub fn parse_project(text: &str) -> anyhow::Result<FabuildProject> {
    let parsed = toml::from_str::<FabuildSeProject>(text)
        .map_err(|e| anyhow::anyhow!(format!("Failed to parse Fabuild project: {e:#?}")))?;

    let project = FabuildProject {
        package: parsed.package,
        version: parsed.version,
        dependencies: parsed
            .dependencies
            .iter()
            .map(|d| (d.0.clone(), d.1.resolve()))
            .collect::<_>(),
        extra: parsed.extra,
    };

    Ok(project)
}

pub fn load_root_project(root: &Path) -> anyhow::Result<FabuildProject> {
    parse_project(&std::fs::read_to_string(root.join("fabuild.toml")).context(
        "loading the root project failed, did you point to a folder without a fabuild project?",
    )?)
}

/// Loads the project and its dependencies into a `FabuildProjectTree`.
/// This is expected to be called on the *root* project only.
pub fn load_project_tree(
    root: &Path,
    project: &FabuildProject,
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

        for (dependency_name, dependency) in &package.dependencies {
            if *dependency_name == parent_name {
                println!("circular dependency found! {dependency_name}");
                break;
            }

            if packages
                .iter()
                .any(|p| p.get_full_name().eq(dependency_name))
            {
                continue;
            }

            let p = parse_project(&registry.resolve_package(dependency_name, &dependency.version)?)
                .context(format!("loading {dependency_name}"))?;
            resolve_dependencies(packages, &p, registry)?;
            let resolved = FabuildResolvedProject {
                package: p.package,
                dependencies: p.dependencies,
                version: p.version,
                ver: dependency.version.clone(),
            };
            packages.push(resolved);
        }

        Ok(())
    }

    let current_toml = std::fs::read_to_string(root.join("fabuild.toml"))
        .context("reading fabuild.toml to evaluate lock validity")?;

    if std::fs::exists(root.join("fabuild.lock")).is_ok_and(|s| s) {
        let content = std::fs::read_to_string(root.join("fabuild.lock"))?;
        let split: Vec<&str> = content.split('\n').collect();

        anyhow::ensure!(
            split
                .first()
                .is_some_and(|s| *s == "# This file is automatically @generated by Fabuild."),
            "Fabuild header is missing!"
        );
        anyhow::ensure!(
            split
                .get(1)
                .is_some_and(|s| *s == "# It is not intended for manual editing."),
            "Fabuild header is missing!"
        );
        anyhow::ensure!(
            split.get(2).is_some_and(|s| s.starts_with("# ")),
            "Fabuild header is missing!"
        );

        // TODO: get the hash of the fabuild.toml not the fabuild.lock lol
        let checksum = split.get(2).ok_or_else(|| unreachable!())?[2..].to_owned();
        let expected = format!("{:#X}", crc32fast::hash(&current_toml.clone().into_bytes()));

        let parsed = toml::from_str(&content);

        if expected == checksum
            && let Ok(parsed) = parsed
        {
            return Ok(parsed);
        }

        println!("hash mismatch, rebuilding lock!");
    }

    let mut packages = vec![];

    resolve_dependencies(&mut packages, project, registry)?;

    let tree = FabuildProjectTree {
        root: project.clone(),
        packages,
    };

    let content = format!(
        "# This file is automatically @generated by Fabuild.\n# It is not intended for manual editing.\n# {:#X}\n{}",
        crc32fast::hash(&current_toml.into_bytes()),
        toml::to_string(&tree)?
    );
    std::fs::write(root.join("fabuild.lock"), content)?;

    Ok(tree)
}
