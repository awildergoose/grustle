use std::{path::Path, str::FromStr};

use crate::{
    jregistry::FabuildJRegistry,
    project::{FabuildProject, FabuildProjectTree},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemArchitecture {
    X86,
    X64,
    Arm64,
    #[default]
    Auto,
}

impl FromStr for SystemArchitecture {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86" => Ok(Self::X86),
            "x64" => Ok(Self::X64),
            "arm64" => Ok(Self::Arm64),
            "auto" => Ok(Self::Auto),
            _ => anyhow::bail!("invalid value for architecture, got: {s}"),
        }
    }
}

impl SystemArchitecture {
    #[must_use]
    pub fn resolve(&self) -> Self {
        if *self != Self::Auto {
            return *self;
        }

        #[cfg(target_arch = "x86_64")]
        return Self::X64;
        #[cfg(target_arch = "x86")]
        return Self::X86;
        #[cfg(any(target_arch = "arm", target_arch = "arm64ec", target_arch = "aarch64"))]
        return Self::Arm64;
    }
}

pub fn generate_log4j_config(root: &Path) -> anyhow::Result<()> {
    Ok(std::fs::write(
        root.join("target").join("log4j.xml"),
        LOG4J_CONFIG,
    )?)
}

pub fn generate_launch_config(
    root: &Path,
    jregistry: &FabuildJRegistry,
    project: &FabuildProject,
) -> anyhow::Result<()> {
    let game_version = project
        .dependencies
        .get("minecraft")
        .ok_or_else(|| anyhow::anyhow!("minecraft is not in the dependency list!"))?;
    let game_client_version = project
        .dependencies
        .get("minecraft-client")
        .ok_or_else(|| anyhow::anyhow!("minecraft-client is not in the dependency list!"))?;
    // TODO: assetIndex here is always 32
    let launch_cfg = format!(
        r"commonProperties
	fabric.development=true
	log4j.configurationFile={}
	log4j2.formatMsgNoLookups=true
	fabric.defaultModDistributionNamespace=official
	fabric.defaultMixinRemapType=static
	fabric.gameJarPath={}
    fabric.classPathGroups={}
	fabric.log.disableAnsi=false
clientArgs
	--assetIndex
	{game_version}-32
	--assetsDir
	{}
clientProperties
	fabric.gameJarPath.client={}
",
        root.join("target")
            .join("log4j.xml")
            .canonicalize()?
            .display(), // log4j.configurationFile
        jregistry
            .resolve_jar("minecraft", game_version, "minecraft.jar")?
            .canonicalize()?
            .display(), // fabric.gameJarPath
        root.join("target")
            .join("classes")
            .canonicalize()?
            .display(), // fabric.classPathGroups
        std::env::var("FABUILD_ASSETS_DIRECTORY")?, // assetsDir
        jregistry
            .resolve_jar(
                "minecraft-client",
                game_client_version,
                "minecraft-client.jar"
            )?
            .canonicalize()?
            .display(), // fabric.gameJarPath.client
    );

    std::fs::write(root.join("target").join("launch.cfg"), launch_cfg)?;

    Ok(())
}

pub fn generate_classpath(
    root: &Path,
    jregistry: &FabuildJRegistry,
    tree: &FabuildProjectTree,
) -> anyhow::Result<()> {
    let entries = tree.gather_classpath(
        root,
        jregistry,
        SystemArchitecture::Auto.resolve(),
        true,
        true,
    )?;
    let classes: Vec<String> = entries.iter().flat_map(|s| s.classes.clone()).collect();
    std::fs::write(root.join("target").join("classpath.tmp"), classes.join(";"))?;

    Ok(())
}

static LOG4J_CONFIG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Configuration status="WARN">
	<Appenders>

		<!--	System out	-->
		<Console name="SysOut" target="SYSTEM_OUT">
			<!-- Filter out the authentication errors when starting in development -->
			<Filters>
				<RegexFilter regex="^Failed to verify authentication$" onMatch="DENY" onMismatch="NEUTRAL"/>
				<RegexFilter regex="^Failed to fetch user properties$" onMatch="DENY" onMismatch="NEUTRAL"/>
				<RegexFilter regex="^Couldn't connect to realms$" onMatch="DENY" onMismatch="NEUTRAL"/>
				<RegexFilter regex="^Failed to fetch Realms feature flags$" onMatch="DENY" onMismatch="NEUTRAL"/>
			</Filters>
			<PatternLayout>
				<LoggerNamePatternSelector defaultPattern="%style{[%d{HH:mm:ss}]}{blue} %highlight{[%t/%level]}{FATAL=red, ERROR=red, WARN=yellow, INFO=green, DEBUG=green, TRACE=blue} %style{(%logger{1})}{cyan} %highlight{%msg%n}{FATAL=red, ERROR=red, WARN=normal, INFO=normal, DEBUG=normal, TRACE=normal}" disableAnsi="${sys:fabric.log.disableAnsi:-true}">
					<!-- Dont show the logger name for minecraft classes-->
					<PatternMatch key="net.minecraft.,com.mojang." pattern="%style{[%d{HH:mm:ss}]}{blue} %highlight{[%t/%level]}{FATAL=red, ERROR=red, WARN=yellow, INFO=green, DEBUG=green, TRACE=blue} %style{(Minecraft)}{cyan} %highlight{%msg{nolookups}%n}{FATAL=red, ERROR=red, WARN=normal, INFO=normal, DEBUG=normal, TRACE=normal}"/>
				</LoggerNamePatternSelector>
			</PatternLayout>
		</Console>

		<!--	Vanilla server gui	-->
		<Queue name="ServerGuiConsole" ignoreExceptions="true">
			<PatternLayout>
				<LoggerNamePatternSelector defaultPattern="[%d{HH:mm:ss} %level] (%logger{1}) %msg{nolookups}%n">
					<!-- Dont show the logger name for minecraft classes-->
					<PatternMatch key="net.minecraft.,com.mojang." pattern="[%d{HH:mm:ss} %level] %msg{nolookups}%n"/>
				</LoggerNamePatternSelector>
			</PatternLayout>
		</Queue>

		<!--	latest.log same as vanilla	-->
		<RollingRandomAccessFile name="LatestFile" fileName="logs/latest.log" filePattern="logs/%d{yyyy-MM-dd}-%i.log.gz">
			<PatternLayout>
				<LoggerNamePatternSelector defaultPattern="[%d{HH:mm:ss}] [%t/%level] (%logger{1}) %msg{nolookups}%n">
					<!-- Dont show the logger name for minecraft classes-->
					<PatternMatch key="net.minecraft.,com.mojang." pattern="[%d{HH:mm:ss}] [%t/%level] (Minecraft) %msg{nolookups}%n"/>
				</LoggerNamePatternSelector>
			</PatternLayout>
			<Policies>
				<TimeBasedTriggeringPolicy />
				<OnStartupTriggeringPolicy />
			</Policies>
		</RollingRandomAccessFile>

		<!--	Debug log file	-->
		<RollingRandomAccessFile name="DebugFile" fileName="logs/debug.log" filePattern="logs/debug-%i.log.gz">
			<PatternLayout pattern="[%d{HH:mm:ss}] [%t/%level] (%logger) %msg{nolookups}%n" />

			<!--	Keep 5 files max	-->
			<DefaultRolloverStrategy max="5" fileIndex="min"/>

			<Policies>
				<SizeBasedTriggeringPolicy size="200MB"/>
				<OnStartupTriggeringPolicy />
			</Policies>

		</RollingRandomAccessFile>
	</Appenders>
	<Loggers>
		<Logger level="${sys:fabric.log.level:-info}" name="net.minecraft"/>
		<Root level="${sys:fabric.log.debug.level:-debug}">
			<AppenderRef ref="DebugFile" level="${sys:fabric.log.debug.level:-debug}"/>
			<AppenderRef ref="SysOut" level="${sys:fabric.log.level:-info}"/>
			<AppenderRef ref="LatestFile" level="${sys:fabric.log.level:-info}"/>
			<AppenderRef ref="ServerGuiConsole" level="${sys:fabric.log.level:-info}"/>
		</Root>
	</Loggers>
</Configuration>"#;
