use clap::Parser;

use crate::{
    codegen::{Codegen, IRCodegen},
    compiler::Compiler,
    lexer::Lexer,
    parser::parser::Parser as RemiParser,
};

use super::cli::args::Args;
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};

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
            args::Command::Run { .. } => todo!(),
            args::Command::Compile { src, out, arch, .. } => match arch {
                Some(_) => todo!(),
                None => {
                    let mut body = String::new();
                    let mut fd = File::open(src)?;
                    fd.read_to_string(&mut body)?;

                    let chars = body.as_str().chars().collect::<Vec<_>>();
                    let lexer = Lexer::new(&chars);
                    let mut parser = RemiParser::new(lexer);
                    let ast = parser.parse()?;
                    let mut compiler = Compiler::new();
                    let stmt = compiler.compile(ast)?;
                    let mut codegen = IRCodegen;
                    let op = codegen
                        .compile(compiler, stmt)
                        .map_err(|err| Box::new(err))?;
                    let mut fd_out = File::create(out)?;
                    fd_out.write(op.as_bytes())?;
                    Ok(())
                }
            },
        }
    }
}
