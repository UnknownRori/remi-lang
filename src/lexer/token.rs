use crate::commons::Loc;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: Loc,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    EOF,
    ParseError,

    Ident(String),
    StringLiteral(String),
    IntLiteral(i64),

    // Puncts
    Plus,
    Minus,
    Star,
    Dot,
    Comma,
    SemiColon,
    Colon,
    Slash,

    Bang,
    Equal,
    NotEqual,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Or,
    And,
    BitOr,
    BitAnd,

    OCurly,
    CCurly,
    OParen,
    CParen,
    OBracket,
    CBracket,

    SpellCard, // Function
    Offer,     // Return
    Eternal,   // Constants Variable
    Vow,       // Mutable Variable
    Invite,    // Foreign Import
    Foreseen,  // If statement
    Otherwise, // else statement
    Until,     // while statement
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::EOF => f.write_str("EOF"),
            TokenKind::ParseError => f.write_str("PARSE ERROR"),
            TokenKind::Ident(str) => f.write_fmt(format_args!("IDENT: {}", str)),
            TokenKind::StringLiteral(str) => f.write_fmt(format_args!("CHAR LITERAL: {}", str)),
            TokenKind::IntLiteral(int) => f.write_fmt(format_args!("INT LITERAL: {}", int)),
            TokenKind::Plus => f.write_str("PLUS"),
            TokenKind::Minus => f.write_str("MINUS"),
            TokenKind::Star => f.write_str("STAR"),
            TokenKind::Dot => f.write_str("DOT"),
            TokenKind::Comma => f.write_str("COMMA"),
            TokenKind::SemiColon => f.write_str("SEMICOLON"),
            TokenKind::Colon => f.write_str("COLON"),
            TokenKind::Slash => f.write_str("SLASH"),
            TokenKind::Bang => f.write_str("BANG"),
            TokenKind::Equal => f.write_str("EQUAL"),
            TokenKind::NotEqual => f.write_str("NOT EQUAL"),
            TokenKind::EqualEqual => f.write_str("EQUAL EQUAL"),
            TokenKind::Less => f.write_str("LESS"),
            TokenKind::LessEqual => f.write_str("LESS EQUAL"),
            TokenKind::Greater => f.write_str("GREATER"),
            TokenKind::GreaterEqual => f.write_str("GREATER EQUAL"),
            TokenKind::Or => f.write_str("OR"),
            TokenKind::And => f.write_str("AND"),
            TokenKind::BitOr => f.write_str("BIT OR"),
            TokenKind::BitAnd => f.write_str("BIT AND"),
            TokenKind::OCurly => f.write_str("OCURLY"),
            TokenKind::CCurly => f.write_str("CCURLY"),
            TokenKind::OParen => f.write_str("OPAREN"),
            TokenKind::CParen => f.write_str("CPAREN"),
            TokenKind::OBracket => f.write_str("OBRACKET"),
            TokenKind::CBracket => f.write_str("CBRACKET"),

            TokenKind::SpellCard => f.write_str("SPELLCARD"),
            TokenKind::Offer => f.write_str("OFFER"),
            TokenKind::Eternal => f.write_str("ETERNAL"),
            TokenKind::Vow => f.write_str("VOW"),
            TokenKind::Invite => f.write_str("INVITE"),
            TokenKind::Foreseen => f.write_str("FORESEEN"),
            TokenKind::Otherwise => f.write_str("OTHERWISE"),
            TokenKind::Until => f.write_str("UNTIL"),
        }
    }
}
