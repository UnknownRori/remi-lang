use crate::value::Value;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Local(usize),
    Literal(Value),
    Constant(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    StackAlloc(usize),
    EternalAssign { index: usize, arg: Arg },
    Label(String),
    Ret(Arg),
}
