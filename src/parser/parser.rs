use super::error::ParseError;

use crate::{
    ast::{Expression, Statement},
    commons::Loc,
    i32,
    lexer::{Lexer, Token, TokenKind},
};

fn get_precedence(token: &TokenKind) -> Option<u8> {
    Some(match token {
        TokenKind::EqualEqual | TokenKind::NotEqual => 1,
        TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => 2,
        TokenKind::Plus | TokenKind::Minus => 3,
        TokenKind::Star | TokenKind::Slash => 4,
        _ => return None,
    })
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            peeked: None,
        }
    }

    fn peek_token(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            self.peeked = self.lexer.next();
        }
        self.peeked.as_ref()
    }

    fn next_token(&mut self, loc: Loc) -> Result<Token, ParseError> {
        if let Some(tok) = self.peeked.take() {
            return Ok(tok);
        }
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

    fn get_indent(&mut self, loc: Loc) -> Result<(String, Loc), ParseError> {
        let name_token = self.next_token(loc)?;
        let new_loc = name_token.loc;
        let name = if let TokenKind::Ident(s) = name_token.kind {
            s
        } else {
            return Err(ParseError::UnexpectedToken {
                found: name_token.kind,
                expected: vec![TokenKind::Ident("".into())],
                loc: name_token.loc,
            });
        };
        Ok((name, new_loc))
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut stmt = vec![];
        while let Some(token) = self.lexer.next() {
            if token.kind == TokenKind::EOF {
                break;
            }

            if let Some(mut subops) = self.parse_statement(token)? {
                stmt.append(&mut subops);
            }
        }
        Ok(stmt)
    }

    fn parse_statement(&mut self, token: Token) -> Result<Option<Vec<Statement>>, ParseError> {
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

    fn parse_ident(&mut self, loc: Loc, _name: String) -> Result<Vec<Statement>, ParseError> {
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
    fn parse_spellcard(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let (name, _) = self.get_indent(loc)?;

        self.expect_kind(loc, TokenKind::OParen)?;
        self.expect_kind(loc, TokenKind::CParen)?;
        let (type_annotation, _) = self.get_indent(loc)?;
        self.expect_kind(loc, TokenKind::OCurly)?;

        let body = self.parse_body(loc)?;
        // TODO : Parse args
        Ok(vec![Statement::SpellCard {
            name,
            args: vec![],
            return_type: Some(type_annotation),
            body,
        }])
    }

    fn parse_eternal(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let (name, next_loc) = self.get_indent(loc)?;
        let token = self.next_token(next_loc)?;
        let mut annotation = None;
        if token.kind == TokenKind::Equal {
            let primary = self.expression(token.loc)?;
            self.expect_kind(token.loc, TokenKind::SemiColon)?;
            return Ok(vec![
                Statement::Eternal {
                    name: name.to_owned(),
                    annotation: None,
                },
                Statement::Assignment {
                    name: name,
                    value: primary,
                },
            ]);
        } else if token.kind == TokenKind::Colon {
            let (annon, next_loc) = self.get_indent(loc)?;
            annotation = Some(annon);

            if token.kind == TokenKind::Equal {
                let primary = self.expression(next_loc)?;
                self.expect_kind(token.loc, TokenKind::SemiColon)?;
                return Ok(vec![
                    Statement::Eternal {
                        name: name.to_owned(),
                        annotation,
                    },
                    Statement::Assignment {
                        name: name,
                        value: primary,
                    },
                ]);
            }
        }

        self.expect_kind(token.loc, TokenKind::SemiColon)?;
        Ok(vec![Statement::Eternal { name, annotation }])
    }

    fn parse_offer(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let primary = self.expression(loc)?;
        self.expect_kind(loc, TokenKind::SemiColon)?;
        Ok(vec![Statement::Offer(primary)])
    }

    fn expression(&mut self, loc: Loc) -> Result<Expression, ParseError> {
        self.bin_expression(0, loc)
    }

    fn bin_expression(&mut self, min_prec: u8, loc: Loc) -> Result<Expression, ParseError> {
        let mut left = self.parse_primary(loc)?;

        loop {
            let op_token = match self.peek_token() {
                Some(tok) if get_precedence(&tok.kind).map_or(false, |p| p >= min_prec) => {
                    tok.clone()
                }
                _ => break,
            };

            let _ = self.next_token(loc)?;
            let right =
                self.bin_expression(get_precedence(&op_token.kind).unwrap() + 1, op_token.loc)?;

            left = Expression::Binary {
                left: Box::new(left),
                op: op_token.kind.try_into().unwrap(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self, loc: Loc) -> Result<Expression, ParseError> {
        let token = self.next_token(loc)?;
        match token.kind {
            TokenKind::IntLiteral(int) => Ok(Expression::Literal(i32!(int as i32))),
            TokenKind::Ident(name) => Ok(Expression::Variable(name)), // TODO : Parse func call
            TokenKind::OParen => {
                let expr = self.expression(token.loc)?;
                self.expect_kind(token.loc, TokenKind::CParen)?;
                Ok(expr)
            }
            other => Err(ParseError::UnexpectedToken {
                found: other,
                expected: vec![
                    TokenKind::IntLiteral(0),
                    TokenKind::Ident("".to_string()),
                    TokenKind::OParen,
                ],
                loc,
            }),
        }
    }

    fn parse_body(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let mut stmt = vec![];
        loop {
            let token = self.next_token(loc)?;
            if token.kind == TokenKind::CCurly {
                break;
            }
            if let Some(mut subops) = self.parse_statement(token)? {
                stmt.append(&mut subops);
            }
        }
        Ok(stmt)
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::BinOp, ast::Expression, i32, value::Value};

    use super::*;

    #[test]
    fn parse_basic_source_code() {
        let body = "
spellcard main() i32 {
    offer 69;
}
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = vec![Statement::SpellCard {
            name: "main".to_owned(),
            args: vec![],
            return_type: Some("i32".to_string()),
            body: vec![Statement::Offer(Expression::Literal(i32!(69)))],
        }];

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

        let expected = vec![Statement::SpellCard {
            name: "main".to_owned(),
            args: vec![],
            return_type: Some("i32".to_string()),
            body: vec![
                Statement::Eternal {
                    name: "foo".to_owned(),
                    annotation: None,
                },
                Statement::Assignment {
                    name: "foo".to_owned(),
                    value: Expression::Literal(i32!(69)),
                },
                Statement::Offer(Expression::Variable("foo".to_owned())),
            ],
        }];

        let lexer = Lexer::new(&chars);
        let mut parser = Parser::new(lexer);
        let ops = parser.parse().expect("Should parse correctly");
        for (i, expect) in expected.iter().enumerate() {
            assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
        }
    }

    #[test]
    fn parse_simple_bin_op() {
        let body = "
spellcard main() i32 {
    eternal foo = 35 + 34;
    offer foo;
}
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = vec![Statement::SpellCard {
            name: "main".to_owned(),
            args: vec![],
            return_type: Some("i32".to_string()),
            body: vec![
                Statement::Eternal {
                    name: "foo".to_owned(),
                    annotation: None,
                },
                Statement::Assignment {
                    name: "foo".to_owned(),
                    value: Expression::Binary {
                        op: BinOp::Add,
                        left: Box::new(Expression::Literal(i32!(35))),
                        right: Box::new(Expression::Literal(i32!(34))),
                    },
                },
                Statement::Offer(Expression::Variable("foo".to_owned())),
            ],
        }];

        let lexer = Lexer::new(&chars);
        let mut parser = Parser::new(lexer);
        let ops = parser.parse().expect("Should parse correctly");
        for (i, expect) in expected.iter().enumerate() {
            assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
        }
    }

    #[test]
    fn parse_with_paren_bin_op() {
        let body = "
spellcard main() i32 {
    eternal foo = 2 * (12 + 4);
    offer foo;
}
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = vec![Statement::SpellCard {
            name: "main".to_owned(),
            args: vec![],
            return_type: Some("i32".to_string()),
            body: vec![
                Statement::Eternal {
                    name: "foo".to_owned(),
                    annotation: None,
                },
                Statement::Assignment {
                    name: "foo".to_owned(),
                    value: Expression::Binary {
                        op: BinOp::Mul,
                        left: Box::new(Expression::Literal(i32!(2))),
                        right: Box::new(Expression::Binary {
                            op: BinOp::Add,
                            left: Box::new(Expression::Literal(i32!(12))),
                            right: Box::new(Expression::Literal(i32!(4))),
                        }),
                    },
                },
                Statement::Offer(Expression::Variable("foo".to_owned())),
            ],
        }];

        let lexer = Lexer::new(&chars);
        let mut parser = Parser::new(lexer);
        let ops = parser.parse().expect("Should parse correctly");
        for (i, expect) in expected.iter().enumerate() {
            assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
        }
    }
}
