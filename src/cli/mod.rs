use clap::Parser;

use super::cli::args::Args;
use std::error::Error;

mod args;

pub struct CLI {
    args: Args,
}

impl CLI {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        match &self.args.command {
            args::Command::Run { src, verbose } => todo!(),
            args::Command::Compile {
                src,
                out,
                arch,
                verbose,
            } => todo!(),
        }
    }
}
