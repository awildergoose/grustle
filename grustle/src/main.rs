#![allow(clippy::missing_errors_doc)]

use arg::Args;

use crate::commands::{ProgramArgs, ProgramSubCommand};

pub mod commands;
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

fn real_main() -> anyhow::Result<()> {
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
    let result = real_main();

    if let Err(ref e) = result {
        let mut con = richrs::console::Console::new();

        let mut text = richrs::text::Text::assemble([
            (
                "error",
                Some(richrs::style::Style::color(richrs::color::Color::Standard(
                    richrs::color::StandardColor::Red,
                ))),
            ),
            (
                ": ",
                Some(richrs::style::Style::color(richrs::color::Color::Standard(
                    richrs::color::StandardColor::White,
                ))),
            ),
        ]);
        text.append_plain(&format!("{e}"));
        con.print_text(&text)?;

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
