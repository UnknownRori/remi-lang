use std::error::Error;

use crate::{commons::Loc, lexer::TokenKind};

pub enum ParseError {
    UnexpectedToken {
        found: TokenKind,
        expected: Vec<TokenKind>,
        loc: Loc,
    },
}

impl Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                found,
                expected,
                loc,
            } => {
                f.write_fmt(format_args!("Found: {} expect: ", found))?;
                for (i, kind) in expected.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_fmt(format_args!("{}", kind))?;
                }
                f.write_fmt(format_args!(" at {}", loc))?;
                Ok(())
            }
        }
    }
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                found,
                expected,
                loc,
            } => {
                f.write_fmt(format_args!("Found: {} expect: ", found))?;
                for (i, kind) in expected.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    f.write_fmt(format_args!("{}", kind))?;
                }
                f.write_fmt(format_args!(" at {}", loc))?;
                Ok(())
            }
        }
    }
}
