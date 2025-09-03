use clap::{Parser, Subcommand};

use crate::target::Target;

#[derive(Parser, Clone, Debug)]
#[command(
    version = "0.0-alpha",
    about = "Remi Programming Language written in Rust with FASM backend",
    long_about = "Remi Programming Language written in Rust with FASM backend"
)]
pub struct Args {
    #[arg(short, long, help = "increase verbosity of output")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    #[command(about = "Run file inside a VM", long_about = "Run file inside a VM ")]
    Run {
        #[arg(short, long)]
        src: String,

        #[arg(short, long, help = "increase verbosity of output")]
        verbose: bool,
    },

    #[command(
        name = "cc",
        about = "Compile file in specified architecture",
        long_about = "Compile specified file using fasm backend for executable or compile into byte code"
    )]
    Compile {
        #[arg(short, long)]
        src: String,

        #[arg(short, long)]
        out: String,

        #[arg(short, long)]
        linker_flag: Option<String>,

        #[arg(short, long)]
        target: Option<Target>,

        #[arg(short, long, help = "increase verbosity of output")]
        verbose: bool,
    },
}
