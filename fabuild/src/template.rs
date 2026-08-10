pub const GITIGNORE: &str = r"/run

# Fabuild
/fabuild.lock
/target

# IDE specific
/.idea

# Datagen
/src/main/generated/.cache
";
// com.azure.azure-json
// name = "azure-json"
// path = "com.azure"
pub const FABUILD_TOML: &str = r#"[package]
name = "{identifier}"
path = "{namespace}"
ver = "1.0.0"

[extra]
display_name = "{display_name}"

[dependencies]
fabric = "{fabric_version}"
fabric-api = "{fabric_api_version}+{minecraft_version}"
minecraft = "{minecraft_version}"
minecraft-client = "{minecraft_version}"

[version]
runtime = ["{identifier}.jar"]
sources = ["{identifier}-sources.jar"]
"#;
pub const FABRIC_MOD_JSON: &str = r#"{
    "schemaVersion": 1,
    "id": "${project.name}",
    "version": "${project.version}",
    "name": "${project.extra.display_name}",
    "description": "This is an example description! Tell everyone what your mod is about!",
    "authors": [
        "Me!"
    ],
    "contact": {
        "homepage": "https://fabricmc.net/",
        "sources": "https://github.com/FabricMC/fabric-example-mod"
    },
    "license": "CC0-1.0",
    "icon": "assets/${project.name}/icon.png",
    "environment": "*",
    "entrypoints": {
        "main": [
            "{package}.{classname}"
        ],
        "client": [
            "{package}.client.{classname}Client"
        ]
    },
    "mixins": [
        "${project.name}.mixins.json",
        {
            "config": "${project.name}.client.mixins.json",
            "environment": "client"
        }
    ],
    "depends": {
        "fabricloader": ">=${project.packages.fabric.version}",
        "minecraft": "~${project.packages.minecraft.version}",
        "java": ">=25",
        "fabric-api": "*"
    }
}"#;
pub const MIXINS_JSON: &str = r#"{
	"required": true,
	"package": "{package}.mixin",
	"compatibilityLevel": "JAVA_25",
	"mixins": [
		
	],
	"injectors": {
		"defaultRequire": 1
	},
	"overwrites": {
		"requireAnnotations": true
	}
}"#;
pub const MIXINS_CLIENT_JSON: &str = r#"{
	"required": true,
	"package": "{package}.client.mixin",
	"compatibilityLevel": "JAVA_25",
	"client": [
		
	],
	"injectors": {
		"defaultRequire": 1
	},
	"overwrites": {
		"requireAnnotations": true
	}
}"#;
pub const MOD_JAVA: &str = r#"package {package};

import net.fabricmc.api.ModInitializer;

import net.minecraft.resources.Identifier;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class {classname} implements ModInitializer {
	public static final String MOD_ID = "{identifier}";

	// This logger is used to write text to the console and the log file.
	// It is considered best practice to use your mod id as the logger's name.
	// That way, it's clear which mod wrote info, warnings, and errors.
	public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

	@Override
	public void onInitialize() {
		// This code runs as soon as Minecraft is in a mod-load-ready state.
		// However, some things (like resources) may still be uninitialized.
		// Proceed with mild caution.

		LOGGER.info("Hello Fabric world!");
	}

	public static Identifier id(String path) {
		return Identifier.fromNamespaceAndPath(MOD_ID, path);
	}
}
"#;
pub const MOD_CLIENT_JAVA: &str = r"package {package}.client;

import net.fabricmc.api.ClientModInitializer;

public class {classname}Client implements ClientModInitializer {
	@Override
	public void onInitializeClient() {
		// This entrypoint is suitable for setting up client-specific logic, such as rendering.
	}
}";

// TODO: This should probably just be a file instead...
pub const TEMPLATE_ICON: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0xAB, 0x00, 0x00, 0x00, 0x80, 0x01, 0x03, 0x00, 0x00, 0x00, 0x4E, 0x4D, 0x0B,
    0xAF, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00, 0xAE, 0xCE, 0x1C, 0xE9, 0x00, 0x00,
    0x00, 0x04, 0x67, 0x41, 0x4D, 0x41, 0x00, 0x00, 0xB1, 0x8F, 0x0B, 0xFC, 0x61, 0x05, 0x00, 0x00,
    0x00, 0x03, 0x50, 0x4C, 0x54, 0x45, 0xFF, 0xFF, 0xFF, 0xA7, 0xC4, 0x1B, 0xC8, 0x00, 0x00, 0x00,
    0x09, 0x70, 0x48, 0x59, 0x73, 0x00, 0x00, 0x0E, 0xC3, 0x00, 0x00, 0x0E, 0xC3, 0x01, 0xC7, 0x6F,
    0xA8, 0x64, 0x00, 0x00, 0x00, 0x1A, 0x49, 0x44, 0x41, 0x54, 0x48, 0xC7, 0xED, 0xC1, 0x31, 0x01,
    0x00, 0x00, 0x00, 0xC2, 0xA0, 0xF5, 0x4F, 0x6D, 0x0B, 0x2F, 0x20, 0x00, 0x00, 0xE0, 0xA4, 0x06,
    0x0B, 0x80, 0x00, 0x01, 0x2F, 0xB1, 0xD9, 0x93, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
    0xAE, 0x42, 0x60, 0x82,
];
