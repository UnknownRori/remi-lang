use crate::value::Value;

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
