use std::path::PathBuf;

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
        long_about = "Compile specified file using fasm backend and use C compiler to compile obj file for executable or compile into byte code"
    )]
    Compile {
        #[arg(required = true)]
        src: Vec<PathBuf>,

        #[arg(short, long)]
        out: Option<String>,

        #[arg(short, long)]
        linker_flag: Option<String>,

        #[arg(short, long)]
        target: Option<Target>,

        #[arg(short = 'S', long, help = "Compile to assembly only")]
        compile_only: bool,

        #[arg(short = 'c', long, help = "Compile to object file only")]
        compile_and_assemble_only: bool,

        #[arg(short, long, help = "increase verbosity of output")]
        verbose: bool,

        #[arg(
            short,
            long,
            help = "Do not remove the temporary file like .asm or .o file (can be use as debugging)"
        )]
        dump: bool,
    },
}
