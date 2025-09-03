use clap::Parser;

use crate::{
    codegen::{Codegen, IRCodegen, JavascriptCodegen, WindowsX86_64},
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
    os::windows::process::CommandExt,
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
        let windows_x86_64: Box<dyn Codegen> = Box::new(WindowsX86_64::new());

        target.insert(Target::IR, ircodegen);
        target.insert(Target::Javascript, jscodegen);
        target.insert(Target::WindowsX86_64, windows_x86_64);

        Self {
            target,
            args: Args::parse(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        match self.args.command.clone() {
            args::Command::Run { .. } => todo!(),
            args::Command::Compile {
                src,
                out,
                target,
                linker_flag,
                ..
            } => {
                let mut body = String::new();
                let mut fd = File::open(src)?;
                fd.read_to_string(&mut body)?;

                let asm_file_name = format!("{}.asm", out);
                let obj_file_name = format!("{}.o", out);

                {
                    match target {
                        Some(target) => match target {
                            Target::LinuxX86_64 | Target::WindowsX86_64 => {
                                {
                                    let mut fd_out = File::create(&asm_file_name)?;
                                    let op = self.compile(target, body)?;
                                    fd_out.write_all(op.as_bytes())?;
                                    fd_out.flush()?;
                                }
                                self.build(asm_file_name, obj_file_name, out, linker_flag)?;
                            }
                            Target::IR | Target::Javascript => {
                                let mut fd_out = File::create(out)?;
                                let op = self.compile(target, body)?;
                                fd_out.write_all(op.as_bytes())?;
                                fd_out.flush()?;
                            }
                            _ => panic!("target is not implemented"),
                        },
                        None => {
                            #[cfg(target_os = "windows")]
                            let arch = target.unwrap_or(crate::target::Target::WindowsX86_64);

                            #[cfg(target_os = "linux")]
                            let arch = arch.unwrap_or(crate::target::Target::LinuxX86_64);

                            {
                                let mut fd_out = File::create(&asm_file_name)?;
                                let op = self.compile(arch, body)?;
                                fd_out.write_all(op.as_bytes())?;
                                fd_out.flush()?;
                            }

                            self.build(asm_file_name, obj_file_name, out, linker_flag)?;
                        }
                    };
                }

                Ok(())
            }
        }
    }

    fn build(
        &mut self,
        asm_file_name: String,
        obj_file_name: String,
        out: String,
        linker_flag: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        {
            let mut a = std::process::Command::new("fasm");
            a.args([asm_file_name, obj_file_name.clone()]);
            a.stdout(std::io::stdout());
            println!(
                "{} {}",
                a.get_program().to_string_lossy(),
                a.get_args()
                    .map(|a| a.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let b = a.output()?;
            unsafe {
                eprintln!("{}", String::from_utf8_unchecked(b.stderr));
            };
        }

        {
            let mut a = std::process::Command::new("gcc");
            a.args([obj_file_name, "-o".to_owned(), out]);
            a.raw_arg(linker_flag.unwrap_or(String::new()));
            a.stdout(std::io::stdout());
            println!(
                "{} {}",
                a.get_program().to_string_lossy(),
                a.get_args()
                    .map(|a| a.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let b = a.output()?;
            unsafe {
                eprintln!("{}", String::from_utf8_unchecked(b.stderr));
            };
        }
        Ok(())
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
