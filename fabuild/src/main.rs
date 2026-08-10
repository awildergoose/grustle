#![allow(clippy::missing_errors_doc)]

pub mod error_parser;
pub mod error_styler;
pub mod jregistry;
pub mod preprocessing;
pub mod project;
pub mod registry;
pub mod tasks;
pub mod template;
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
pub struct ProgramNewSubCommand {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,

    #[arg(short, long, required)]
    pub namespace: String,
    #[arg(short, long, required)]
    pub identifier: String,
    #[arg(short, long, required)]
    pub classname: String,
    #[arg(short, long, required)]
    pub display_name: String,

    #[arg(short, long, default_value = "\"26.2\".to_string()")]
    pub minecraft_version: String,
    #[arg(short, long, default_value = "\"0.19.3\".to_string()")]
    pub fabric_version: String,
    #[arg(short, long, default_value = "\"0.155.2\".to_string()")]
    pub fabric_api_version: String,
}

#[derive(Args, Debug)]
pub struct ProgramRunSubCommand {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
    #[arg(long)]
    pub server: bool,
    #[arg(long)]
    pub datagen: bool,
}

#[derive(Args, Debug)]
pub struct ProgramSourcesSubCommand {
    #[arg(long, default_value = "PathBuf::from(\"../example\")")]
    pub root: PathBuf,
    #[arg(
        long,
        default_value = "std::thread::available_parallelism().map(|n| (n.get() / 2) as u32).unwrap_or(2)"
    )]
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
    New(ProgramNewSubCommand),
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

fn fake_main() -> anyhow::Result<()> {
    // TODO: improve this
    let binding = std::env::args().skip(1).collect::<Vec<String>>();
    let args = binding
        .iter()
        .map(std::string::String::as_str)
        .collect::<Vec<&str>>();
    let args = ProgramArgs::from_args(args)
        .map_err(|e| anyhow::anyhow!("failed to parse command: {e:?}"))?;

    match args.cmd {
        ProgramSubCommand::New(args) => tasks::new::run(&args),
        ProgramSubCommand::Build(args) => tasks::build::run(&args),
        ProgramSubCommand::Run(args) => tasks::run::run(&args),
        ProgramSubCommand::Jar(args) => tasks::jar::run(&args),
        ProgramSubCommand::Classpath(args) => tasks::classpath::run(&args),
        ProgramSubCommand::Aw(args) => tasks::aw::run(&args),
        ProgramSubCommand::Intellij(args) => tasks::intellij::run(&args),
        ProgramSubCommand::Sources(args) => tasks::sources::run(&args),
    }
}

fn main() -> anyhow::Result<()> {
    let result = fake_main();

    if let Err(ref e) = result {
        let mut con = richrs::console::Console::new();

        con.print(&format!("[red]error[white]: {e}"))?;

        if cfg!(debug_assertions) {
            let backtrace_status = e.backtrace().status();

            if backtrace_status.eq(&std::backtrace::BacktraceStatus::Captured) {
                println!("{}", e.backtrace());
            } else if backtrace_status.eq(&std::backtrace::BacktraceStatus::Disabled) {
                con.print(
                    "[cyan]help[white]: set [green]RUST_BACKTRACE=1[white] to see the backtrace",
                )?;
            }
        }
    }

    Ok(())
}
