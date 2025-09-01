use crate::commons::Loc;

use super::token::{Token, TokenKind};

pub struct Lexer<'a> {
    content: &'a [char],
    loc: Loc,
}

const KEYWORDS: [(&str, TokenKind); 6] = [
    ("spellcard", TokenKind::SpellCard),
    ("offer", TokenKind::Offer),
    ("eternal", TokenKind::Eternal),
    ("vow", TokenKind::Vow),
    ("and", TokenKind::Eternal),
    ("or", TokenKind::Vow),
];

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self {
            content,
            loc: Loc { row: 1, column: 1 },
        }
    }

    fn loc_add(&mut self, row: usize, column: usize) {
        self.loc.column += column;
        self.loc.row += row;
    }

    fn skip(&mut self, n: usize) {
        self.loc.column += n;
        self.content = &self.content[n..];
    }

    fn skip_only(&mut self, n: usize) {
        self.content = &self.content[n..];
    }

    fn skip_n_return(&mut self, n: usize, kind: TokenKind) -> Token {
        let loc = self.loc;
        self.skip(n);
        let temp = Token { kind, loc };
        temp
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> (String, Loc)
    where
        P: FnMut(&char) -> bool,
    {
        let loc = self.loc;
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.loc.column += n;
        let chop = self.chop(n).iter().collect::<String>();
        (chop, loc)
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.content.is_empty() {
            return None;
        }

        return match self.content[0] {
            '+' => Some(self.skip_n_return(1, TokenKind::Plus)),
            '-' => Some(self.skip_n_return(1, TokenKind::Minus)),
            '*' => Some(self.skip_n_return(1, TokenKind::Star)),
            '.' => Some(self.skip_n_return(1, TokenKind::Dot)),
            ',' => Some(self.skip_n_return(1, TokenKind::Comma)),
            ';' => Some(self.skip_n_return(1, TokenKind::SemiColon)),
            ':' => Some(self.skip_n_return(1, TokenKind::Colon)),
            '{' => Some(self.skip_n_return(1, TokenKind::OCurly)),
            '}' => Some(self.skip_n_return(1, TokenKind::CCurly)),
            '(' => Some(self.skip_n_return(1, TokenKind::OParen)),
            ')' => Some(self.skip_n_return(1, TokenKind::CParen)),
            '[' => Some(self.skip_n_return(1, TokenKind::OBracket)),
            ']' => Some(self.skip_n_return(1, TokenKind::CBracket)),
            '/' => {
                if self.content[1] != '/' {
                    return Some(self.skip_n_return(1, TokenKind::Slash));
                }
                self.skip(2);

                let _ = self.chop_while(|a| *a != '\n');
                self.next_token()
            }
            '!' => {
                if self.content[1] == '=' {
                    return Some(self.skip_n_return(2, TokenKind::NotEqual));
                }

                Some(self.skip_n_return(1, TokenKind::Bang))
            }
            '=' => {
                if self.content[1] == '=' {
                    return Some(self.skip_n_return(2, TokenKind::EqualEqual));
                }

                Some(self.skip_n_return(1, TokenKind::Equal))
            }
            '>' => {
                if self.content[1] == '=' {
                    return Some(self.skip_n_return(2, TokenKind::GreaterEqual));
                }

                Some(self.skip_n_return(1, TokenKind::Greater))
            }
            '<' => {
                if self.content[1] == '=' {
                    return Some(self.skip_n_return(2, TokenKind::LessEqual));
                }

                Some(self.skip_n_return(1, TokenKind::Less))
            }
            '|' => {
                if self.content[1] == '|' {
                    return Some(self.skip_n_return(2, TokenKind::Or));
                }
                Some(self.skip_n_return(1, TokenKind::BitOr))
            }
            '&' => {
                if self.content[1] == '|' {
                    return Some(self.skip_n_return(2, TokenKind::And));
                }
                Some(self.skip_n_return(1, TokenKind::BitAnd))
            }
            '0'..='9' => {
                let (chop, loc) = self.chop_while(|a| a.is_digit(10));

                Some(Token {
                    kind: TokenKind::IntLiteral(chop.parse().unwrap()),
                    loc,
                })
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let (chop, loc) = self.chop_while(|a| a.is_alphanumeric() || *a == '_');

                for (keyword, kind) in KEYWORDS {
                    if keyword == chop {
                        return Some(Token { loc, kind });
                    }
                }

                Some(Token {
                    kind: TokenKind::Ident(chop),
                    loc,
                })
            }
            '"' => {
                self.skip_only(1);
                let (chop, loc) = self.chop_while(|a| *a != '"');
                self.skip_only(1);
                self.loc_add(0, 2);
                Some(Token {
                    kind: TokenKind::StringLiteral(chop),
                    loc,
                })
            }
            ' ' | '\t' => {
                self.skip(1);
                self.next_token()
            }
            '\n' => {
                self.skip(1);
                self.loc.column = 1;
                self.loc.row += 1;
                self.next_token()
            }
            '\r' => {
                self.skip_only(1);
                self.next_token()
            }
            _ => None,
        };
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_puncts() {
        let body = "[] () {} | & = == != > >= < <= + - * / . ; :";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = [
            TokenKind::OBracket,
            TokenKind::CBracket,
            TokenKind::OParen,
            TokenKind::CParen,
            TokenKind::OCurly,
            TokenKind::CCurly,
            TokenKind::BitOr,
            TokenKind::BitAnd,
            TokenKind::Equal,
            TokenKind::EqualEqual,
            TokenKind::NotEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Dot,
            TokenKind::SemiColon,
            TokenKind::Colon,
        ];

        let mut lexer = Lexer::new(&chars);

        for kind in expected {
            let token = lexer.next().expect("it should have token");
            assert_eq!(token.kind, kind);
        }
    }

    #[test]
    fn parse_keyword() {
        let body = "spellcard offer eternal vow and or";
        let chars = body.chars().collect::<Vec<_>>();
        let mut lexer = Lexer::new(&chars);

        for (_, kind) in KEYWORDS {
            let token = lexer.next().expect("it should have token");
            assert_eq!(token.kind, kind);
        }
    }

    #[test]
    fn parse_ident() {
        let body = "main";
        let chars = body.chars().collect::<Vec<_>>();
        let mut lexer = Lexer::new(&chars);
        let token = lexer.next().expect("It should have 1 token");
        assert_eq!(token.kind, TokenKind::Ident("main".to_owned()));
        assert_eq!(token.loc.row, 1);
        assert_eq!(token.loc.column, 1);
        assert_eq!(lexer.loc.column, 5);
    }

    #[test]
    fn parse_string_literal() {
        let body = "\"Hello, World\"";
        let chars = body.chars().collect::<Vec<_>>();
        let mut lexer = Lexer::new(&chars);
        let token = lexer.next().expect("It should have 1 token");
        assert_eq!(
            token.kind,
            TokenKind::StringLiteral("Hello, World".to_owned())
        );
        assert_eq!(token.loc.row, 1);
        assert_eq!(token.loc.column, 1);
        assert_eq!(lexer.loc.column, 15);
    }

    #[test]
    fn parse_string_literal_multiple() {
        let body = "\"Hello, World\" \"Hi\"";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = [
            (
                Loc::new(1, 1),
                TokenKind::StringLiteral("Hello, World".to_owned()),
            ),
            (Loc::new(16, 1), TokenKind::StringLiteral("Hi".to_owned())),
        ];

        let mut lexer = Lexer::new(&chars);

        for (loc, kind) in expected {
            let token = lexer.next().expect("it should have token");
            assert_eq!(token.kind, kind);
            assert_eq!(token.loc.column, loc.column);
            assert_eq!(token.loc.row, loc.row);
        }
    }

    #[test]
    fn parse_basic_source_code() {
        let body = "
spellcard main() i32 {
    eternal ret = 69;
    offer ret;
}
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = [
            (Loc::new(1, 2), TokenKind::SpellCard),
            (Loc::new(11, 2), TokenKind::Ident("main".to_owned())),
            (Loc::new(15, 2), TokenKind::OParen),
            (Loc::new(16, 2), TokenKind::CParen),
            (Loc::new(18, 2), TokenKind::Ident("i32".to_owned())),
            (Loc::new(22, 2), TokenKind::OCurly),
            (Loc::new(5, 3), TokenKind::Eternal),
            (Loc::new(13, 3), TokenKind::Ident("ret".to_string())),
            (Loc::new(17, 3), TokenKind::Equal),
            (Loc::new(19, 3), TokenKind::IntLiteral(69)),
            (Loc::new(21, 3), TokenKind::SemiColon),
            (Loc::new(5, 4), TokenKind::Offer),
            (Loc::new(11, 4), TokenKind::Ident("ret".to_string())),
            (Loc::new(14, 4), TokenKind::SemiColon),
            (Loc::new(1, 5), TokenKind::CCurly),
        ];

        let mut lexer = Lexer::new(&chars);

        for (loc, kind) in expected {
            let token = lexer.next().expect("it should have token");
            assert_eq!(token.kind, kind);
            assert_eq!(token.loc.column, loc.column);
            assert_eq!(token.loc.row, loc.row);
        }
    }

    #[test]
    fn skip_comment() {
        let body = "
// Hi There
// Yoo
main
        ";
        let chars = body.chars().collect::<Vec<_>>();

        let expected = [(Loc::new(1, 4), TokenKind::Ident("main".to_owned()))];

        let mut lexer = Lexer::new(&chars);

        for (loc, kind) in expected {
            let token = lexer.next().expect("it should have token");
            assert_eq!(token.kind, kind);
            assert_eq!(token.loc.column, loc.column);
            assert_eq!(token.loc.row, loc.row);
        }
    }
}
