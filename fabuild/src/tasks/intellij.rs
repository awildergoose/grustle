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
    // <?xml version="1.0" encoding="UTF-8"?>
    // <module type="JAVA_MODULE" version="4">
    //   <component name="NewModuleRootManager" inherit-compiler-output="true">
    //     <exclude-output />
    //     <content url="file://$MODULE_DIR$">
    //       <sourceFolder url="file://$MODULE_DIR$/src/client/java" isTestSource="false" />
    //       <sourceFolder url="file://$MODULE_DIR$/src/client/resources" type="java-resource" />
    //       <sourceFolder url="file://$MODULE_DIR$/src/main/java" isTestSource="false" />
    //       <sourceFolder url="file://$MODULE_DIR$/src/main/resources" type="java-resource" />
    //     </content>
    //     <orderEntry type="inheritedJdk" />
    //     <orderEntry type="sourceFolder" forTests="false" />
    //     <orderEntry type="module-library" scope="PROVIDED">
    //       <library>
    //         <CLASSES>
    //           <root url="jar://$MODULE_DIR$/../../.fabuild/net.fabricmc.fabric-api/fabric-game-rule-api-v1/4.0.8+46a6d00c9e/fabric-game-rule-api-v1-4.0.8+46a6d00c9e.jar!/" />
    //         </CLASSES>
    //         <JAVADOC />
    //         <SOURCES />
    //       </library>
    //     </orderEntry>
    //   </component>
    // </module>
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
        <orderEntry type="inheritedJdk" />
        <orderEntry type="sourceFolder" forTests="false" />
"#
    .to_string();

    let project = load_root_project(&root)?;
    let registry = load_default_registry();
    let jregistry = load_default_jregistry();
    let tree = load_project_tree(&root, project, &registry)?;

    let entries =
        tree.gather_classpath(&root, &jregistry, SystemArchitecture::Auto.resolve(), false)?;

    for entry in &entries {
        let mut classes = String::new();
        let mut sources = String::new();

        for class in &entry.classes {
            write!(&mut classes, "<root url=\"jar://{class}!/\" />\n\t\t\t\t")?;
        }

        for source in &entry.sources {
            write!(&mut sources, "<root url=\"jar://{source}!/\" />\n\t\t\t\t")?;
        }

        classes = classes.trim().to_owned();
        sources = sources.trim().to_owned();

        write!(
            &mut out,
            r#"<orderEntry type="module-library" scope="PROVIDED">
            <library>
                <CLASSES>
                    {classes}
                </CLASSES>
                <JAVADOC />
                <SOURCES>
                    {sources}
                </SOURCES>
            </library>
        </orderEntry>"#
        )?;
    }

    out = out.trim().to_owned();

    out += r"    </component>
</module>";

    println!("{out}");

    Ok(())
}
