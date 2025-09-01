use std::collections::HashMap;

use super::error::ParseError;

use crate::{
    commons::Loc,
    i32,
    lexer::{Lexer, Token, TokenKind},
    op::{Arg, Op},
    value::{DataType, Value},
};

use super::alloc::ScopeAlloc;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    scope: ScopeAlloc,
    vars: HashMap<String, usize>,
}

/// TODO : This should emit AST instead of OP
impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            scope: ScopeAlloc::default(),
            vars: HashMap::new(),
        }
    }

    fn next_token(&mut self, loc: Loc) -> Result<Token, ParseError> {
        self.lexer.next().ok_or(ParseError::UnexpectedToken {
            found: TokenKind::EOF,
            expected: vec![],
            loc,
        })
    }

    fn expect_kind(&mut self, loc: Loc, expected: TokenKind) -> Result<Token, ParseError> {
        let token = self.next_token(loc)?;
        if token.kind == expected {
            Ok(token)
        } else {
            Err(ParseError::UnexpectedToken {
                found: token.kind,
                expected: vec![expected],
                loc: token.loc,
            })
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Op>, ParseError> {
        let mut ops = Vec::new();
        while let Some(token) = self.lexer.next() {
            if token.kind == TokenKind::EOF {
                break;
            }
            if let Some(mut subops) = self.parse_token(token)? {
                ops.append(&mut subops);
            }
        }
        Ok(ops)
    }

    fn parse_token(&mut self, token: Token) -> Result<Option<Vec<Op>>, ParseError> {
        match token.kind {
            TokenKind::Ident(name) => self.parse_ident(token.loc, name).map(Some),
            TokenKind::SpellCard => self.parse_spellcard(token.loc).map(Some),
            TokenKind::Offer => self.parse_offer(token.loc).map(Some),
            TokenKind::Eternal => self.parse_eternal(token.loc).map(Some),
            TokenKind::Vow => todo!("Type alias"),
            TokenKind::EOF => Ok(None),
            _ => Err(ParseError::UnexpectedToken {
                found: token.kind,
                expected: vec![TokenKind::Ident("".into())],
                loc: token.loc,
            }),
        }
    }

    fn parse_ident(&mut self, loc: Loc, name: String) -> Result<Vec<Op>, ParseError> {
        let token = self.next_token(loc)?;
        match token.kind {
            TokenKind::OParen => todo!("function call"),
            TokenKind::Equal => todo!("assignment"),
            _ => Err(ParseError::UnexpectedToken {
                found: token.kind,
                expected: vec![TokenKind::OParen, TokenKind::Equal],
                loc: token.loc,
            }),
        }
    }

    fn parse_spellcard(&mut self, loc: Loc) -> Result<Vec<Op>, ParseError> {
        let name_token = self.next_token(loc)?;
        let name = if let TokenKind::Ident(s) = name_token.kind {
            s
        } else {
            return Err(ParseError::UnexpectedToken {
                found: name_token.kind,
                expected: vec![TokenKind::Ident("".into())],
                loc: name_token.loc,
            });
        };

        self.expect_kind(loc, TokenKind::OParen)?;
        self.expect_kind(loc, TokenKind::CParen)?;
        _ = self.next_token(loc)?; // TODO : Handle it's type
        self.expect_kind(loc, TokenKind::OCurly)?;

        let mut ops = vec![Op::Label(name)];
        ops.append(&mut self.parse_body(loc)?);
        Ok(ops)
    }

    fn parse_offer(&mut self, loc: Loc) -> Result<Vec<Op>, ParseError> {
        let token = self.next_token(loc)?;
        match token.kind {
            TokenKind::Ident(name) => {
                self.expect_kind(token.loc, TokenKind::SemiColon)?;
                if let Some(&id) = self.vars.get(&name) {
                    Ok(vec![Op::Ret(Arg::Local(id))])
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: TokenKind::Ident(name),
                        expected: vec![],
                        loc: token.loc,
                    })
                }
            }
            TokenKind::IntLiteral(int) => {
                self.expect_kind(token.loc, TokenKind::SemiColon)?;
                Ok(vec![Op::Ret(Arg::Literal(Value::I32(int as i32)))])
            }
            TokenKind::StringLiteral(s) => {
                self.expect_kind(token.loc, TokenKind::SemiColon)?;
                todo!("return string literal {s}")
            }
            _ => Err(ParseError::UnexpectedToken {
                found: token.kind,
                expected: vec![
                    TokenKind::Ident("".into()),
                    TokenKind::IntLiteral(0),
                    TokenKind::StringLiteral("".into()),
                ],
                loc: token.loc,
            }),
        }
    }

    fn parse_eternal(&mut self, loc: Loc) -> Result<Vec<Op>, ParseError> {
        let name_token = self.next_token(loc)?;
        let name = if let TokenKind::Ident(n) = name_token.kind {
            n
        } else {
            return Err(ParseError::UnexpectedToken {
                found: name_token.kind,
                expected: vec![TokenKind::Ident("".into())],
                loc: name_token.loc,
            });
        };

        let id = self.scope.alloc(DataType::I32);
        self.vars.insert(name.clone(), id);

        let next = self.next_token(name_token.loc)?;
        match next.kind {
            TokenKind::Equal => {
                let value_token = self.next_token(next.loc)?;
                match value_token.kind {
                    TokenKind::IntLiteral(int) => {
                        self.expect_kind(next.loc, TokenKind::SemiColon)?;
                        Ok(vec![
                            Op::StackAlloc(1),
                            Op::EternalAssign {
                                index: id,
                                arg: Arg::Literal(Value::I32(int as i32)),
                            },
                        ])
                    }
                    TokenKind::StringLiteral(_) => todo!("eternal assign string"),
                    _ => Err(ParseError::UnexpectedToken {
                        found: value_token.kind,
                        expected: vec![TokenKind::IntLiteral(0)],
                        loc: value_token.loc,
                    }),
                }
            }
            TokenKind::SemiColon => Ok(vec![Op::StackAlloc(1)]),
            _ => Err(ParseError::UnexpectedToken {
                found: next.kind,
                expected: vec![TokenKind::Equal, TokenKind::SemiColon],
                loc: next.loc,
            }),
        }
    }

    fn parse_body(&mut self, loc: Loc) -> Result<Vec<Op>, ParseError> {
        let mut ops = vec![];
        loop {
            let token = self.next_token(loc)?;
            if token.kind == TokenKind::CCurly {
                break;
            }
            if let Some(mut inner) = self.parse_token(token)? {
                ops.append(&mut inner);
            }
        }
        Ok(ops)
    }
}

#[cfg(test)]
mod test {
    use crate::{i32, value::Value};

    use super::*;

    #[test]
    fn parse_basic_source_code() {
        let body = "
spellcard main() i32 {
    offer 69;
}
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = vec![
            Op::Label("main".to_string()),
            Op::Ret(Arg::Literal(Value::I32(69))),
        ];

        let lexer = Lexer::new(&chars);
        let mut parser = Parser::new(lexer);
        let ops = parser.parse().expect("Should parse correctly");
        for (i, expect) in expected.iter().enumerate() {
            assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
        }
    }

    #[test]
    fn parse_basic_source_code_with_eternal() {
        let body = "
spellcard main() i32 {
    eternal foo = 69;
    offer foo;
}
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = vec![
            Op::Label("main".to_string()),
            Op::StackAlloc(1),
            Op::EternalAssign {
                index: 0,
                arg: Arg::Literal(i32!(69)),
            },
            Op::Ret(Arg::Local(0)),
        ];

        let lexer = Lexer::new(&chars);
        let mut parser = Parser::new(lexer);
        let ops = parser.parse().expect("Should parse correctly");
        for (i, expect) in expected.iter().enumerate() {
            assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
        }
    }
}
