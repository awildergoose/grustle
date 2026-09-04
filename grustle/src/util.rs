use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    jregistry::GrustleJRegistry,
    project::{GrustleProject, GrustleProjectTree},
};
use anyhow::Context;

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

#[must_use]
pub fn get_target_folder(root: &Path) -> PathBuf {
    root.join("target")
}

#[must_use]
pub fn get_target_classes_folder(root: &Path) -> PathBuf {
    get_target_folder(root).join("classes")
}

#[must_use]
pub fn get_target_launch_folder(root: &Path) -> PathBuf {
    get_target_folder(root).join("launch")
}

#[must_use]
pub fn get_target_classtweakers_folder(root: &Path) -> PathBuf {
    get_target_folder(root).join("classtweakers")
}

#[must_use]
pub fn get_target_aw_folder(root: &Path) -> PathBuf {
    get_target_folder(root).join("aw")
}

#[must_use]
pub fn get_target_sources_file(root: &Path) -> PathBuf {
    get_target_folder(root).join("sources")
}

#[must_use]
pub fn get_target_common_classpath_file(root: &Path) -> PathBuf {
    get_target_folder(root).join("classpath.tmp")
}

#[must_use]
pub fn get_target_client_classpath_file(root: &Path) -> PathBuf {
    get_target_folder(root).join("classpath_client.tmp")
}

#[must_use]
pub fn get_target_qt_file(root: &Path) -> PathBuf {
    get_target_aw_folder(root).join("qt.jar")
}

#[must_use]
pub fn get_target_cachyflower_file(root: &Path) -> PathBuf {
    get_target_aw_folder(root).join("cachyflower.jar")
}

#[must_use]
pub fn get_run_folder(root: &Path) -> PathBuf {
    root.join("run")
}

pub fn generate_log4j_config(root: &Path) -> anyhow::Result<()> {
    Ok(std::fs::write(
        get_target_launch_folder(root).join("log4j.xml"),
        LOG4J_CONFIG,
    )?)
}

pub fn generate_launch_config(
    root: &Path,
    jregistry: &GrustleJRegistry,
    project: &GrustleProject,
) -> anyhow::Result<()> {
    let launch = get_target_launch_folder(root);

    let game_version = project
        .dependencies
        .get("minecraft")
        .ok_or_else(|| anyhow::anyhow!("minecraft is not in the dependency list!"))?
        .version
        .clone();
    let game_client_version = project
        .dependencies
        .get("minecraft-client")
        .ok_or_else(|| anyhow::anyhow!("minecraft-client is not in the dependency list!"))?
        .version
        .clone();

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
        launch.join("log4j.xml").canonicalize()?.display(), // log4j.configurationFile
        jregistry
            .resolve_file(root, "minecraft", &game_version, "minecraft.jar")?
            .canonicalize()?
            .display(), // fabric.gameJarPath
        get_target_classes_folder(root).canonicalize()?.display(), // fabric.classPathGroups
        std::env::var("GRUSTLE_ASSETS_DIRECTORY")
            .context("GRUSTLE_ASSETS_DIRECTORY is not set!")?, // assetsDir
        jregistry
            .resolve_file(
                root,
                "minecraft-client",
                &game_client_version,
                "minecraft-client.jar"
            )?
            .canonicalize()?
            .display(), // fabric.gameJarPath.client
    );

    std::fs::create_dir_all(&launch)?;
    std::fs::write(launch.join("launch.cfg"), launch_cfg)?;

    Ok(())
}

pub fn generate_client_classpath(
    root: &Path,
    jregistry: &GrustleJRegistry,
    tree: &GrustleProjectTree,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(get_target_classes_folder(root))?;
    let entries = tree.gather_classpath(
        root,
        jregistry,
        SystemArchitecture::Auto.resolve(),
        true,
        true,
    )?;
    let classes: Vec<String> = entries.iter().flat_map(|s| s.classes.clone()).collect();
    std::fs::write(get_target_client_classpath_file(root), classes.join(";"))?;

    Ok(())
}

pub fn generate_common_classpath(
    root: &Path,
    jregistry: &GrustleJRegistry,
    tree: &GrustleProjectTree,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(get_target_classes_folder(root))?;
    let game_client_version = tree
        .root
        .dependencies
        .get("minecraft-client")
        .ok_or_else(|| anyhow::anyhow!("minecraft-client is not in the dependency list!"))?
        .version
        .clone();

    let entries = tree.gather_classpath(
        root,
        jregistry,
        SystemArchitecture::Auto.resolve(),
        true,
        true,
    )?;
    let mut classes: Vec<String> = entries.iter().flat_map(|s| s.classes.clone()).collect();
    let minecraft_client = jregistry
        .resolve_file(
            root,
            "minecraft-client",
            &game_client_version,
            "minecraft-client.jar",
        )?
        .display()
        .to_string();
    classes = classes
        .iter()
        .filter(|s| **s != minecraft_client)
        .cloned()
        .collect::<Vec<String>>();
    std::fs::write(get_target_common_classpath_file(root), classes.join(";"))?;

    Ok(())
}

pub fn split_jobs<I>(iter: I, n: usize) -> Vec<Vec<I::Item>>
where
    I: IntoIterator,
{
    let items: Vec<_> = iter.into_iter().collect();
    let total_len = items.len();

    if n == 0 || total_len == 0 {
        return vec![];
    }

    let chunk_size = total_len / n;

    if chunk_size == 0 {
        return items.into_iter().map(|item| vec![item]).collect();
    }

    let mut chunks = Vec::with_capacity(n);
    let mut drain = items.into_iter();

    for _ in 0..(n - 1) {
        let chunk: Vec<_> = drain.by_ref().take(chunk_size).collect();
        chunks.push(chunk);
    }

    let last_chunk: Vec<_> = drain.collect();
    chunks.push(last_chunk);

    chunks
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
