use std::{collections::HashMap, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    jregistry::GrustleJRegistry,
    registry::GrustleRegistry,
    util::{SystemArchitecture, get_target_classes_folder},
};

// Spaghetti
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrustlePackage {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub ver: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GrustlePackageVersionNatives {
    pub x86: Vec<String>,
    pub x64: Vec<String>,
    pub arm64: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GrustlePackageVersion {
    pub runtime: Vec<String>,
    pub sources: Vec<String>,
    #[serde(default)]
    pub natives: GrustlePackageVersionNatives,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrustleDependency {
    pub version: String,
    #[serde(default)]
    pub embedded: bool,
    #[serde(default)]
    pub location: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GrustleSeDependency {
    Versioned(String),
    Full(GrustleDependency),
}

impl GrustleSeDependency {
    #[must_use]
    pub fn resolve(&self) -> GrustleDependency {
        match self {
            Self::Versioned(v) => GrustleDependency {
                version: v.clone(),
                embedded: false,
                location: String::new(),
            },
            Self::Full(d) => d.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrustleSeProject {
    pub package: GrustlePackage,
    pub dependencies: HashMap<String, GrustleSeDependency>,
    pub version: GrustlePackageVersion,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrustleProject {
    pub package: GrustlePackage,
    pub dependencies: HashMap<String, GrustleDependency>,
    pub version: GrustlePackageVersion,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl GrustleProject {
    #[must_use]
    pub fn get_full_name(&self) -> String {
        if self.package.path.is_empty() {
            return self.package.name.clone();
        }

        format!("{}.{}", self.package.path, self.package.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrustleResolvedProject {
    pub package: GrustlePackage,
    pub dependencies: HashMap<String, GrustleDependency>,
    pub version: GrustlePackageVersion,
    pub ver: String,
}

impl GrustleResolvedProject {
    #[must_use]
    pub fn get_full_name(&self) -> String {
        if self.package.path.is_empty() {
            return self.package.name.clone();
        }

        format!("{}.{}", self.package.path, self.package.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrustleProjectTree {
    pub root: GrustleProject,
    pub packages: Vec<GrustleResolvedProject>,
}

pub struct ClasspathEntry {
    pub classes: Vec<String>,
    pub sources: Vec<String>,
}

impl GrustleProjectTree {
    /// Returns (Vec<Classpath>, Vec<Sourcepath>)
    pub fn gather_classpath(
        &self,
        root: &Path,
        jregistry: &GrustleJRegistry,
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

pub fn parse_project(text: &str) -> anyhow::Result<GrustleProject> {
    let parsed = toml::from_str::<GrustleSeProject>(text)
        .map_err(|e| anyhow::anyhow!(format!("Failed to parse Grustle project: {e:#?}")))?;

    let project = GrustleProject {
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

pub fn load_root_project(root: &Path) -> anyhow::Result<GrustleProject> {
    parse_project(&std::fs::read_to_string(root.join("grustle.toml")).context(
        "loading the root project failed, did you point to a folder without a Grustle project?",
    )?)
}

/// Loads the project and its dependencies into a `GrustleProjectTree`.
/// This is expected to be called on the *root* project only.
pub fn load_project_tree(
    root: &Path,
    project: &GrustleProject,
    registry: &GrustleRegistry,
) -> anyhow::Result<GrustleProjectTree> {
    fn resolve_dependencies(
        packages: &mut Vec<GrustleResolvedProject>,
        package: &GrustleProject,
        registry: &GrustleRegistry,
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
            let resolved = GrustleResolvedProject {
                package: p.package,
                dependencies: p.dependencies,
                version: p.version,
                ver: dependency.version.clone(),
            };
            packages.push(resolved);
        }

        Ok(())
    }

    let current_toml = std::fs::read_to_string(root.join("grustle.toml"))
        .context("reading grustle.toml to evaluate lock validity")?;

    if std::fs::exists(root.join("grustle.lock")).is_ok_and(|s| s) {
        let content = std::fs::read_to_string(root.join("grustle.lock"))?;
        let split: Vec<&str> = content.split('\n').collect();

        anyhow::ensure!(
            split
                .first()
                .is_some_and(|s| *s == "# This file is automatically @generated by Grustle."),
            "Grustle header is missing!"
        );
        anyhow::ensure!(
            split
                .get(1)
                .is_some_and(|s| *s == "# It is not intended for manual editing."),
            "Grustle header is missing!"
        );
        anyhow::ensure!(
            split.get(2).is_some_and(|s| s.starts_with("# ")),
            "Grustle header is missing!"
        );

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

    let tree = GrustleProjectTree {
        root: project.clone(),
        packages,
    };

    let content = format!(
        "# This file is automatically @generated by Grustle.\n# It is not intended for manual editing.\n# {:#X}\n{}",
        crc32fast::hash(&current_toml.into_bytes()),
        toml::to_string(&tree)?
    );
    std::fs::write(root.join("grustle.lock"), content)?;

    Ok(tree)
}
