use super::error::ParseError;

use crate::{
    ast::{Expression, FunctionArgs, Statement, UnaryOp},
    commons::Loc,
    i32,
    lexer::{Lexer, Token, TokenKind},
    string,
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

    fn expect_many_kind_but_no_consume(
        &mut self,
        loc: Loc,
        expected: Vec<TokenKind>,
    ) -> Result<bool, ParseError> {
        let token = self.peek_token();
        for i in expected {
            if token.is_none() {
                return Err(ParseError::UnexpectedToken {
                    found: TokenKind::EOF,
                    expected: vec![TokenKind::Ident("".to_string()), TokenKind::CParen],
                    loc,
                });
            }

            if token.unwrap().kind == i {
                return Ok(true);
            }
        }
        Ok(false)
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
            TokenKind::Invite => self.parse_invite(token.loc).map(Some),
            TokenKind::Foreseen => self.parse_foreseen(token.loc).map(Some),
            TokenKind::Until => self.parse_until(token.loc).map(Some),
            TokenKind::Vow => self.parse_vow(token.loc).map(Some),
            TokenKind::EOF => Ok(None),
            _ => Err(ParseError::UnexpectedToken {
                found: token.kind,
                expected: vec![TokenKind::Ident("".into())],
                loc: token.loc,
            }),
        }
    }

    fn parse_ident(&mut self, loc: Loc, name: String) -> Result<Vec<Statement>, ParseError> {
        let token = self.next_token(loc)?;
        match token.kind {
            TokenKind::OParen => {
                let args = self.parse_call(token.loc)?;
                self.expect_kind(token.loc, TokenKind::CParen)?;
                self.expect_kind(token.loc, TokenKind::SemiColon)?;
                Ok(vec![Statement::Expression(Expression::Call {
                    function: name,
                    args,
                })])
            }
            TokenKind::Equal => {
                let value = self.expression(loc)?;
                self.expect_kind(token.loc, TokenKind::SemiColon)?;
                Ok(vec![Statement::Assignment { name, value }])
            }
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
        let mut args = vec![];
        loop {
            let (name_param, loc) = match self.peek_token() {
                Some(token) => match token.kind.clone() {
                    TokenKind::Ident(name) => (name, token.loc.clone()),
                    TokenKind::CParen => break,
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            found: token.kind.clone(),
                            expected: vec![TokenKind::Ident("".to_string()), TokenKind::CParen],
                            loc: token.loc,
                        });
                    }
                },
                None => {
                    return Err(ParseError::UnexpectedToken {
                        found: TokenKind::EOF,
                        expected: vec![TokenKind::Ident("".to_string()), TokenKind::CParen],
                        loc,
                    });
                }
            };
            let tok = self.next_token(loc)?;
            self.expect_kind(tok.loc, TokenKind::Colon)?;
            let (annotation, _) = self.get_indent(tok.loc)?;
            args.push(FunctionArgs {
                name: name_param,
                annotation,
            });

            if self.expect_many_kind_but_no_consume(loc, vec![TokenKind::Comma])? {
                self.expect_kind(loc, TokenKind::Comma)?;
            }
        }
        self.expect_kind(loc, TokenKind::CParen)?;
        let (type_annotation, _) = self.get_indent(loc)?;
        self.expect_kind(loc, TokenKind::OCurly)?;

        let body = self.parse_body(loc)?;
        // TODO : Parse args
        Ok(vec![Statement::SpellCard {
            name,
            args,
            return_type: Some(type_annotation),
            body,
        }])
    }

    fn parse_vow(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let (name, next_loc) = self.get_indent(loc)?;
        let token = self.next_token(next_loc)?;
        let mut annotation = None;
        if token.kind == TokenKind::Equal {
            let primary = self.expression(token.loc)?;
            self.expect_kind(token.loc, TokenKind::SemiColon)?;
            return Ok(vec![
                Statement::Vow {
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
                    Statement::Vow {
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
        Ok(vec![Statement::Vow { name, annotation }])
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
        let loc = match self.peek_token().clone() {
            Some(token) => match token.kind {
                TokenKind::SemiColon => loc,
                _ => {
                    let primary = self.expression(loc)?;
                    self.expect_kind(loc, TokenKind::SemiColon)?;
                    return Ok(vec![Statement::Offer(Some(primary))]);
                }
            },
            None => todo!(),
        };

        self.next_token(loc)?;
        Ok(vec![Statement::Offer(None)])
    }

    fn parse_invite(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let (name, new_loc) = self.get_indent(loc)?;
        self.expect_kind(new_loc, TokenKind::SemiColon)?;
        Ok(vec![Statement::Invite { name }])
    }

    fn parse_foreseen(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let condition = self.expression(loc)?;
        self.expect_kind(loc, TokenKind::OCurly)?;
        let then_branch = self.parse_body(loc)?;
        let mut else_branch = None;
        match self.peek_token() {
            Some(token) => match token.kind {
                TokenKind::Otherwise => {
                    let token = self.next_token(loc)?;
                    self.expect_kind(token.loc, TokenKind::OCurly)?;
                    else_branch = Some(self.parse_body(token.loc)?);
                }
                _ => {}
            },
            None => {}
        }

        Ok(vec![Statement::Foreseen {
            condition,
            then_branch,
            else_branch,
        }])
    }

    fn parse_until(&mut self, loc: Loc) -> Result<Vec<Statement>, ParseError> {
        let condition = self.expression(loc)?;
        self.expect_kind(loc, TokenKind::OCurly)?;
        let then_branch = self.parse_body(loc)?;

        Ok(vec![Statement::Until {
            condition,
            body: then_branch,
        }])
    }

    fn parse_bang(&mut self, loc: Loc) -> Result<Expression, ParseError> {
        let right = self.parse_primary(loc)?;
        Ok(Expression::Unary {
            op: UnaryOp::Not,
            arg: Box::new(right),
        })
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
            TokenKind::StringLiteral(str) => Ok(Expression::Literal(string!(str))),
            TokenKind::Bang => Ok(self.parse_bang(token.loc)?),
            TokenKind::Ident(name) => {
                let args = match self.peek_token() {
                    Some(tok) if tok.kind == TokenKind::OParen => {
                        self.expect_kind(loc, TokenKind::OParen)?;
                        let call = self.parse_call(loc)?;
                        self.expect_kind(loc, TokenKind::CParen)?;
                        call
                    }
                    _ => return Ok(Expression::Variable(name)),
                };
                Ok(Expression::Call {
                    function: name,
                    args,
                })
            }
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

    fn parse_call(&mut self, loc: Loc) -> Result<Vec<Expression>, ParseError> {
        let mut stmt = vec![];
        loop {
            let loc = match self.peek_token() {
                Some(token) => match token.kind.clone() {
                    TokenKind::CParen => break,
                    _ => token.loc.clone(),
                },
                None => {
                    return Err(ParseError::UnexpectedToken {
                        found: TokenKind::EOF,
                        expected: vec![TokenKind::CParen],
                        loc,
                    });
                }
            };
            let expr = self.expression(loc)?;
            stmt.push(expr);
            if self.expect_many_kind_but_no_consume(loc, vec![TokenKind::Comma])? {
                self.expect_kind(loc, TokenKind::Comma)?;
            }
        }
        Ok(stmt)
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
