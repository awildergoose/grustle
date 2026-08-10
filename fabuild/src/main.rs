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
    #[arg(default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
}

#[derive(Args, Debug)]
pub struct ProgramClasspathArgs {
    #[arg(default_value = "PathBuf::from(\"../example\")")]
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
    Run(ProgramEmptySubCommand),
    Jar(ProgramEmptySubCommand),
    Classpath(ProgramClasspathArgs),
    Aw(ProgramEmptySubCommand),
    Intellij(ProgramEmptySubCommand),
    Sources(ProgramEmptySubCommand),
}

#[derive(Args, Debug)]
struct ProgramArgs {
    #[arg(sub)]
    cmd: ProgramSubCommand,
}

fn main() -> anyhow::Result<()> {
    // TODO: improve this
    let args = ProgramArgs::from_text(&std::env::args().skip(1).collect::<Vec<String>>().join(" "))
        .map_err(|_| anyhow::anyhow!("failed to parse command"))?;

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
