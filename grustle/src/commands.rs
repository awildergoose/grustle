use std::path::PathBuf;

use arg::Args;

use crate::util::SystemArchitecture;

#[cfg(debug_assertions)]
const ROOT: &str = "../example";
#[cfg(not(debug_assertions))]
const ROOT: &str = "./";

#[derive(Args, Debug)]
pub struct ProgramEmptySubCommand {
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
    pub root: PathBuf,

    #[arg(long)]
    pub profile: String,
    #[arg(long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct ProgramNewSubCommand {
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
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
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
    pub root: PathBuf,

    #[arg(long)]
    pub server: bool,
    #[arg(long)]
    pub datagen: bool,

    #[arg(long)]
    pub profile: String,
    #[arg(long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct ProgramSourcesSubCommand {
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
    pub root: PathBuf,

    #[arg(
        long,
        default_value = "std::thread::available_parallelism().map(|n| (n.get() / 2) as u32).unwrap_or(2)"
    )]
    pub threads: u32,

    #[arg(long)]
    pub profile: String,
    #[arg(long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct ProgramClasspathArgs {
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
    pub root: PathBuf,

    #[arg(short, long, default_value = "false")]
    pub sources: bool,
    #[arg(long, default_value = "SystemArchitecture::Auto")]
    pub arch: SystemArchitecture,

    #[arg(long)]
    pub profile: String,
    #[arg(long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct ProgramBuildSubCommand {
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
    pub root: PathBuf,

    #[arg(long)]
    pub profile: String,
    #[arg(long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub struct ProgramJarSubCommand {
    #[arg(long, default_value = "PathBuf::from(ROOT)")]
    pub root: PathBuf,

    #[arg(long)]
    pub profile: String,
    #[arg(long)]
    pub release: bool,
}

#[derive(Args, Debug)]
pub enum ProgramSubCommand {
    New(ProgramNewSubCommand),
    Build(ProgramBuildSubCommand),
    Run(ProgramRunSubCommand),
    Jar(ProgramJarSubCommand),
    Classpath(ProgramClasspathArgs),
    Aw(ProgramEmptySubCommand),
    Intellij(ProgramEmptySubCommand),
    Sources(ProgramSourcesSubCommand),
}

#[derive(Args, Debug)]
pub struct ProgramArgs {
    #[arg(sub)]
    pub cmd: ProgramSubCommand,
}

pub trait ProfilefulArg {
    fn profile(&self) -> &str;
}

macro_rules! profilefuls {
    ($($name: tt),*) => {
        $(
            impl ProfilefulArg for $name {
                fn profile(&self) -> &str {
                    if !self.release {
                        if self.profile != "" {
                            &self.profile
                        } else {
                            "debug"
                        }
                    } else {
                        "release"
                    }
                }
            }
        )*
    };
}

profilefuls!(
    ProgramEmptySubCommand,
    ProgramClasspathArgs,
    ProgramBuildSubCommand,
    ProgramJarSubCommand,
    ProgramSourcesSubCommand,
    ProgramRunSubCommand
);
