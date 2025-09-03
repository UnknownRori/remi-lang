use clap::Parser;

use crate::{
    codegen::{Codegen, IRCodegen, JavascriptCodegen},
    compiler::Compiler,
    lexer::Lexer,
    parser::parser::Parser as RemiParser,
    target::Target,
};

use super::cli::args::Args;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{Read, Write},
};

mod args;

pub struct CLI {
    args: Args,
    target: HashMap<Target, Box<dyn Codegen>>,
}

impl CLI {
    pub fn new() -> Self {
        let mut target: HashMap<Target, Box<dyn Codegen>> = HashMap::new();

        let ircodegen: Box<dyn Codegen> = Box::new(IRCodegen);
        let jscodegen: Box<dyn Codegen> = Box::new(JavascriptCodegen::new());
        target.insert(Target::IR, ircodegen);
        target.insert(Target::Javascript, jscodegen);

        Self {
            target,
            args: Args::parse(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        match self.args.command.clone() {
            args::Command::Run { .. } => todo!(),
            args::Command::Compile { src, out, arch, .. } => {
                let mut body = String::new();
                let mut fd = File::open(src)?;
                fd.read_to_string(&mut body)?;

                let mut fd_out = File::create(out)?;
                match arch {
                    Some(arch) => {
                        let op = self.compile(arch, body)?;
                        fd_out.write(op.as_bytes())?;
                        Ok(())
                    }
                    None => {
                        let op = self.compile(Target::IR, body)?;
                        fd_out.write(op.as_bytes())?;
                        Ok(())
                    }
                }
            }
        }
    }

    fn compile(&mut self, arch: Target, src_code: String) -> Result<String, Box<dyn Error>> {
        let chars = src_code.as_str().chars().collect::<Vec<_>>();
        let lexer = Lexer::new(&chars);
        let mut parser = RemiParser::new(lexer);
        let ast = parser.parse()?;
        let mut compiler = Compiler::new();
        let stmt = compiler.compile(ast)?;

        let mut codegen = self.target.get_mut(&arch);
        let codegen = codegen.take().unwrap();
        let out = codegen.compile(compiler, stmt)?;
        Ok(out)
    }
}
