use crate::{lexer::TokenKind, value::Value};

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,

    Equal,
    Greater,
    Less,
}

impl TryFrom<TokenKind> for BinOp {
    type Error = ();

    fn try_from(value: TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Plus => Ok(BinOp::Add),
            TokenKind::Minus => Ok(BinOp::Sub),
            TokenKind::Star => Ok(BinOp::Mul),
            TokenKind::Slash => Ok(BinOp::Div),
            TokenKind::EqualEqual => Ok(BinOp::Equal),
            TokenKind::Greater => Ok(BinOp::Greater),
            TokenKind::Less => Ok(BinOp::Less),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Binary {
        op: BinOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        function: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionArgs {
    pub name: String,
    pub annotation: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Invite {
        name: String,
    },
    Eternal {
        name: String,
        annotation: Option<String>,
    },
    Vow {
        name: String,
        value: Expression,
        annotation: Option<String>,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    Foreseen {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Until {
        condition: Expression,
        body: Vec<Statement>,
    },
    SpellCard {
        name: String,
        args: Vec<FunctionArgs>,
        return_type: Option<String>,
        body: Vec<Statement>,
    },
    Offer(Expression),
}
