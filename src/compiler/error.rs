use crate::commons::Loc;
use std::{error::Error, fmt::Write};

#[derive(Debug)]
pub enum CompilerError {
    TypeMissmatch {
        statement: String,
        expected: String,
        found: String,
        loc: Loc,
    },
    UndefinedVariable {
        found: String,
        loc: Loc,
    },
    UnknownFunction {
        found: String,
        loc: Loc,
    },
}

impl Error for CompilerError {}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::TypeMissmatch {
                statement,
                expected,
                found,
                loc,
            } => f.write_fmt(format_args!(
                "Type missmatch on {}, expected {}, but found {}, at {}",
                statement, expected, found, loc
            )),
            CompilerError::UndefinedVariable { found, loc } => {
                f.write_fmt(format_args!("Undefined variable of {} at {}", found, loc))
            }
            CompilerError::UnknownFunction { found, loc } => f.write_fmt(format_args!(
                "Undefined function symbol of {} at {}",
                found, loc
            )),
        }
    }
}
