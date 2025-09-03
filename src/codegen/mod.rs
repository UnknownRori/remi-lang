mod ir;
mod javascript;

use std::{error::Error, fmt::Write};

pub use ir::*;
pub use javascript::*;

use crate::{compiler::Compiler, op::Op};

pub trait Codegen {
    fn compile(&mut self, compiler: Compiler, stmt: Vec<Op>) -> Result<String, CodegenError>;
}

#[derive(Debug)]
pub enum CodegenError {
    Unsupported { op: Op },
}

impl Error for CodegenError {}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::Unsupported { op } => f.write_fmt(format_args!("{}", op)),
        }
    }
}
