package com.example;

import net.fabricmc.api.ModInitializer;

import net.minecraft.resources.Identifier;
import net.minecraft.client.Minecraft;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TemplateMod implements ModInitializer {
	public static final String MOD_ID = "template-mod";

	// This logger is used to write text to the console and the log file.
	// It is considered best practice to use your mod id as the logger's name.
	// That way, it's clear which mod wrote info, warnings, and errors.
	public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    // #define FABRIC

	@Override
	public void onInitialize() {
		// This code runs as soon as Minecraft is in a mod-load-ready state.
		// However, some things (like resources) may still be uninitialized.
		// Proceed with mild caution.

//#if version(minecraft) >= 26.0
        LOGGER.info("Hello >=26.0 world!");
//#endif

//#if version(minecraft) < 26.0
        LOGGER.info("Hello <26.0 world!");
//#endif

//#ifdef FABRIC
		LOGGER.info("Hello Fabric world!");
//#endif
//#ifdef HEROBRINE
        LOGGER.info("BOO");
//#endif
	}

	public static Identifier id(String path) {
		return Identifier.fromNamespaceAndPath(MOD_ID, path);
	}
}
