use build::{build_ast, build_exe, build_obj};
use clap::Parser;

use crate::{
    codegen::{Codegen, IRCodegen, JavascriptCodegen, LinuxX86_64, WindowsX86_64},
    compiler::Compiler,
    op::Op,
    target::Target,
};

use super::cli::args::Args;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

mod args;
mod build;

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
        let linux: Box<dyn Codegen> = Box::new(LinuxX86_64::new());

        target.insert(Target::IR, ircodegen);
        target.insert(Target::Javascript, jscodegen);
        target.insert(Target::WindowsX86_64, windows_x86_64);
        target.insert(Target::LinuxX86_64, linux);

        Self {
            target,
            args: Args::parse(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        match self.args.command.clone() {
            args::Command::Run { .. } => todo!(),
            args::Command::Compile {
                target,
                out,
                src,
                verbose,
                linker_flag,
                compile_only,
                compile_and_assemble_only,
                dump,
            } => {
                let src_file = src
                    .iter()
                    .map(|path| -> Result<(String, PathBuf), Box<dyn Error>> {
                        let mut buf = String::new();
                        let mut file = File::open(path).map_err(|err| Box::new(err))?;
                        file.read_to_string(&mut buf).map_err(|err| Box::new(err))?;
                        Ok((buf, path.clone()))
                    })
                    .map(|a| a.expect("Fail to read a file"))
                    .collect::<Vec<_>>();

                let ast = src_file
                    .iter()
                    .map(|(code, path)| (build_ast(code.clone()).unwrap(), path))
                    .collect::<Vec<_>>();

                let out = out.unwrap_or(String::from("a.out"));
                let mut obj_temp = vec![];
                let mut asm_temp = vec![];
                let mut args = vec![String::from("-no-pie")];
                if let Some(linker_flag) = linker_flag {
                    let split = linker_flag.split(" ");
                    for i in split {
                        args.push(String::from(i));
                    }
                }
                match target {
                    Some(arch) => match arch {
                        Target::WindowsX86_64 | Target::LinuxX86_64 => {
                            if compile_only {
                                self.compile_only(&out, arch, ast)?;
                            } else if compile_and_assemble_only {
                                self.assemble_only(&out, arch, ast, &mut asm_temp, verbose)?;
                            } else {
                                self.build_exe(
                                    &out,
                                    arch,
                                    ast,
                                    &args,
                                    &mut asm_temp,
                                    &mut obj_temp,
                                    verbose,
                                )?;
                            }
                        }
                        Target::Javascript => todo!(),
                        Target::IR => {
                            for ((op, compiler), original_path) in ast {
                                let asm_file =
                                    format!("{}.ir.asm", original_path.to_string_lossy());
                                let codegen = self
                                    .target
                                    .get_mut(&arch)
                                    .take()
                                    .expect("Target is not yet implemented");

                                let asm = codegen.compile(compiler, op)?;
                                {
                                    let mut file = File::create(&asm_file)?;
                                    file.write_all(asm.as_bytes())?;
                                }
                            }
                        }
                        Target::Bytecode => todo!(),
                    },
                    None => {
                        #[cfg(target_os = "windows")]
                        let arch = target.unwrap_or(crate::target::Target::WindowsX86_64);

                        #[cfg(target_os = "linux")]
                        let arch = target.unwrap_or(crate::target::Target::LinuxX86_64);

                        if compile_only {
                            self.compile_only(&out, arch, ast)?;
                        } else if compile_and_assemble_only {
                            self.assemble_only(&out, arch, ast, &mut asm_temp, verbose)?;
                        } else {
                            self.build_exe(
                                &out,
                                arch,
                                ast,
                                &args,
                                &mut asm_temp,
                                &mut obj_temp,
                                verbose,
                            )?;
                        }
                    }
                }

                self.clear_temp(obj_temp, asm_temp, dump);
                Ok(())
            }
        }
    }

    fn compile_only(
        &mut self,
        out: &str,
        arch: Target,
        mut ast: Vec<((Vec<Op>, Compiler), &PathBuf)>,
    ) -> Result<(), Box<dyn Error>> {
        let ((op, compiler), _) = ast.pop().expect("No inputs");
        let codegen = self
            .target
            .get_mut(&arch)
            .take()
            .expect("Target is not yet implemented");

        let asm = codegen.compile(compiler, op)?;
        {
            let mut file = File::create(out)?;
            file.write_all(asm.as_bytes())?;
        }
        Ok(())
    }

    fn assemble_only(
        &mut self,
        out: &str,
        arch: Target,
        mut ast: Vec<((Vec<Op>, Compiler), &PathBuf)>,
        asm_temp: &mut Vec<String>,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        let ((op, compiler), original_path) = ast.pop().expect("No inputs");
        let asm_file = format!("{}.asm", original_path.to_string_lossy());
        let codegen = self
            .target
            .get_mut(&arch)
            .take()
            .expect("Target is not yet implemented");

        let asm = codegen.compile(compiler, op)?;
        {
            let mut file = File::create(&asm_file)?;
            file.write_all(asm.as_bytes())?;
        }
        build_obj(&asm_file, out, verbose).map_err(|err| Box::new(err))?;
        asm_temp.push(asm_file);
        Ok(())
    }

    fn build_exe(
        &mut self,
        out: &str,
        arch: Target,
        ast: Vec<((Vec<Op>, Compiler), &PathBuf)>,
        args: &Vec<String>,
        asm_temp: &mut Vec<String>,
        obj_temp: &mut Vec<String>,
        verbose: bool,
    ) -> Result<(), Box<dyn Error>> {
        for ((op, compiler), original_path) in ast {
            let asm_file = format!("{}.asm", original_path.to_string_lossy());
            let obj_file = format!("{}.o", original_path.to_string_lossy());
            let codegen = self
                .target
                .get_mut(&arch)
                .take()
                .expect("Target is not yet implemented");

            let asm = codegen.compile(compiler, op)?;
            {
                let mut file = File::create(&asm_file)?;
                file.write_all(asm.as_bytes())?;
            }
            build_obj(&asm_file, &obj_file, verbose).map_err(|err| Box::new(err))?;

            asm_temp.push(asm_file);
            obj_temp.push(obj_file);
        }

        build_exe(
            "gcc",
            obj_temp.as_slice(),
            out.as_ref(),
            args.as_slice(),
            verbose,
        )?;
        Ok(())
    }

    fn clear_files(&self, temp: Vec<String>) {
        for i in temp {
            let _ = std::fs::remove_file(i);
        }
    }

    fn clear_temp(&self, obj_temp: Vec<String>, asm_temp: Vec<String>, dump: bool) {
        if !dump {
            self.clear_files(obj_temp);
            self.clear_files(asm_temp);
        }
    }
}
