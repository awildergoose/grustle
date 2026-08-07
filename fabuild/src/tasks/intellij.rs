use std::fmt::Write;
use std::path::PathBuf;

use crate::{
    ProgramEmptySubCommand,
    jregistry::load_default_jregistry,
    project::{load_project_tree, load_root_project},
    registry::load_default_registry,
    util::SystemArchitecture,
};

pub fn run(_args: &ProgramEmptySubCommand) -> anyhow::Result<()> {
    let root = PathBuf::from("../example");
    let mut out = r#"<?xml version="1.0" encoding="UTF-8"?>
    <module type="JAVA_MODULE" version="4">
    <component name="NewModuleRootManager" inherit-compiler-output="true">
    <exclude-output />
    <content url="file://$MODULE_DIR$">
    <sourceFolder url="file://$MODULE_DIR$/src/client/java" isTestSource="false" />
    <sourceFolder url="file://$MODULE_DIR$/src/client/resources" type="java-resource" />
    <sourceFolder url="file://$MODULE_DIR$/src/main/java" isTestSource="false" />
    <sourceFolder url="file://$MODULE_DIR$/src/main/resources" type="java-resource" />
    </content>
        "#
    .to_string();

    let project = load_root_project(&root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry();
    let tree = load_project_tree(&root, project, &registry)?;

    let sources_classpath =
        tree.gather_classpath(&root, &jregistry, true, SystemArchitecture::Auto.resolve())?;

    for source in &sources_classpath {
        write!(
            &mut out,
            r#"<content url="jar://{source}!/">
            <sourceFolder url="jar://{source}!/" isTestSource="false" />
        </content>
        "#
        )?;
    }

    out = out.trim().to_owned();

    out += r#"
        <orderEntry type="inheritedJdk" />
        <orderEntry type="sourceFolder" forTests="false" />
    </component>
</module>"#;

    println!("{out}");

    Ok(())
}
