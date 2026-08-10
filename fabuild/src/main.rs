#![allow(clippy::missing_errors_doc)]

pub mod error_parser;
pub mod error_styler;
pub mod jregistry;
pub mod preprocessing;
pub mod project;
pub mod registry;
pub mod tasks;
pub mod tweaker;
pub mod util;

use std::path::PathBuf;

use arg::Args;

use crate::util::SystemArchitecture;

#[derive(Args, Debug)]
pub struct ProgramEmptySubCommand {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
}

#[derive(Args, Debug)]
pub struct ProgramRunSubCommand {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
    // little hack i hate
    #[arg(long, default_value = "true")]
    pub client: bool,
    #[arg(long, default_value = "false")]
    pub server: bool,
}

#[derive(Args, Debug)]
pub struct ProgramSourcesSubCommand {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
    #[arg(long, default_value = "4")]
    pub threads: u32,
}

#[derive(Args, Debug)]
pub struct ProgramClasspathArgs {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
    #[arg(short, long, default_value = "false")]
    pub sources: bool,
    #[arg(long, default_value = "SystemArchitecture::Auto")]
    pub arch: SystemArchitecture,
}

#[derive(Args, Debug)]
enum ProgramSubCommand {
    Init(ProgramEmptySubCommand),
    Build(ProgramEmptySubCommand),
    Run(ProgramRunSubCommand),
    Jar(ProgramEmptySubCommand),
    Classpath(ProgramClasspathArgs),
    Aw(ProgramEmptySubCommand),
    Intellij(ProgramEmptySubCommand),
    Sources(ProgramSourcesSubCommand),
}

#[derive(Args, Debug)]
struct ProgramArgs {
    #[arg(sub)]
    cmd: ProgramSubCommand,
}

fn main() -> anyhow::Result<()> {
    // TODO: improve this
    let binding = std::env::args().skip(1).collect::<Vec<String>>();
    let args = binding
        .iter()
        .map(std::string::String::as_str)
        .collect::<Vec<&str>>();
    let args = ProgramArgs::from_args(args)
        .map_err(|e| anyhow::anyhow!("failed to parse command: {e:?}"))?;

    match args.cmd {
        ProgramSubCommand::Init(args) => tasks::init::run(&args),
        ProgramSubCommand::Build(args) => tasks::build::run(&args),
        ProgramSubCommand::Run(args) => tasks::run::run(&args),
        ProgramSubCommand::Jar(args) => tasks::jar::run(&args),
        ProgramSubCommand::Classpath(args) => tasks::classpath::run(&args),
        ProgramSubCommand::Aw(args) => tasks::aw::run(&args),
        ProgramSubCommand::Intellij(args) => tasks::intellij::run(&args),
        ProgramSubCommand::Sources(args) => tasks::sources::run(&args),
    }
}
