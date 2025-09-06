use std::{error::Error, process::Command};

use crate::{compiler::Compiler, lexer::Lexer, op::Op, parser::parser::Parser as RemiParser};

pub enum BuildCommandResult {
    BuildFailed { message: String },
}

impl std::error::Error for BuildCommandResult {}
impl std::fmt::Debug for BuildCommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildCommandResult::BuildFailed { message } => {
                f.write_fmt(format_args!("Build failed: {}", message))
            }
        }
    }
}
impl std::fmt::Display for BuildCommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildCommandResult::BuildFailed { message } => {
                f.write_fmt(format_args!("Build failed: {}", message))
            }
        }
    }
}

fn print_cmd(cmd: &Command) {
    eprintln!(
        "{} {}\n",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
}

pub fn build_ast(src_code: String) -> Result<(Vec<Op>, Compiler), Box<dyn Error>> {
    let chars = src_code.as_str().chars().collect::<Vec<_>>();
    let lexer = Lexer::new(&chars);
    let mut parser = RemiParser::new(lexer);
    let ast = parser.parse()?;
    let mut compiler = Compiler::new();
    let stmt = compiler.compile(ast)?;

    Ok((stmt, compiler))
}

pub fn build_obj(asm_file: &str, obj_file: &str, log: bool) -> Result<(), BuildCommandResult> {
    let mut cmd = std::process::Command::new("fasm");
    cmd.args([asm_file, obj_file]);
    if log {
        cmd.stdout(std::io::stdout());
        cmd.stderr(std::io::stderr());
        print_cmd(&cmd);
    }
    let out = cmd
        .output()
        .map_err(|err| BuildCommandResult::BuildFailed {
            message: format!("Failed to compile to obj file {}", err.to_string()),
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(out.stderr.as_slice());
        return Err(BuildCommandResult::BuildFailed {
            message: format!("Failed to compile to obj file: \n{}", stderr),
        });
    }

    Ok(())
}

pub fn build_exe(
    cc: &str,
    obj_file: &[String],
    exe_file: &str,
    args: &[String],
    log: bool,
) -> Result<(), BuildCommandResult> {
    let mut cmd = std::process::Command::new(cc);
    cmd.args(["-o", exe_file]);
    for i in obj_file {
        cmd.arg(i);
    }
    for i in args {
        cmd.arg(i);
    }

    if log {
        cmd.stdout(std::io::stdout());
        cmd.stderr(std::io::stderr());
        print_cmd(&cmd);
    }
    let out = cmd
        .output()
        .map_err(|err| BuildCommandResult::BuildFailed {
            message: format!("Failed to compile to obj file {}", err.to_string()),
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(out.stderr.as_slice());
        return Err(BuildCommandResult::BuildFailed {
            message: format!("Failed to compile to exe file: \n{}", stderr),
        });
    }

    Ok(())
}
