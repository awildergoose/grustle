use crate::{
    commands::ProgramNewSubCommand,
    template::{
        FABRIC_MOD_JSON, GITIGNORE, GRUSTLE_TOML, MIXINS_CLIENT_JSON, MIXINS_JSON, MOD_CLIENT_JAVA,
        MOD_JAVA, TEMPLATE_ICON,
    },
};

pub fn run(args: &ProgramNewSubCommand) -> anyhow::Result<()> {
    let root = &args.root;

    if std::fs::exists(root.join("grustle.toml"))? {
        anyhow::bail!("a Grustle project already exists at that folder!");
    }

    macro_rules! rpath {
        ($path:expr) => {
            root.join(format!($path))
        };
    }

    macro_rules! mkdirs {
        ($($path:expr),*) => {
            $(
                std::fs::create_dir_all(rpath!($path))?;
            )*
        };
    }

    macro_rules! writef {
        ($path:expr, $text:expr, $($key:expr),*) => {
            std::fs::write(
                rpath!($path),
                $text$(.replace(&format!("{{{}}}", stringify!($key)), $key))*
            )?
        };
        ($path:expr, $text:expr) => {
            writef!($path, $text,)
        };
    }

    let (
        namespace,
        identifier,
        classname,
        display_name,
        minecraft_version,
        fabric_version,
        fabric_api_version,
    ) = (
        &args.namespace,
        &args.identifier,
        &args.classname,
        &args.display_name,
        &args.minecraft_version,
        &args.fabric_version,
        &args.fabric_api_version,
    );

    let package = &format!("{namespace}.{identifier}");
    let fpackage = package.replace('.', "/");

    mkdirs!(
        "src/main/java/{fpackage}/mixin/",
        "src/client/java/{fpackage}/client/mixin/",
        "src/main/resources/assets/{identifier}/",
        "src/client/resources/"
    );

    writef!(".gitignore", GITIGNORE);
    writef!(
        "grustle.toml",
        GRUSTLE_TOML,
        identifier,
        namespace,
        display_name,
        minecraft_version,
        fabric_version,
        fabric_api_version
    );
    writef!(
        "src/main/resources/fabric.mod.json",
        FABRIC_MOD_JSON,
        package,
        classname
    );

    writef!(
        "src/main/resources/{identifier}.mixins.json",
        MIXINS_JSON,
        package
    );
    writef!(
        "src/client/resources/{identifier}.client.mixins.json",
        MIXINS_CLIENT_JSON,
        package
    );

    writef!(
        "src/main/java/{fpackage}/{classname}.java",
        MOD_JAVA,
        package,
        classname,
        identifier
    );
    writef!(
        "src/client/java/{fpackage}/client/{classname}Client.java",
        MOD_CLIENT_JAVA,
        package,
        classname
    );

    writef!(
        "src/main/resources/assets/{identifier}/icon.png",
        TEMPLATE_ICON,
    );

    println!("Created new project at {}!", root.canonicalize()?.display());

    Ok(())
}
