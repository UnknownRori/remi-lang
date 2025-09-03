use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Local(usize),
    Literal(Value),
    DataOffset(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    StackAlloc(usize),
    Invite { name: String },
    EternalAssign { offset: usize, arg: Arg },
    Label(String),
    Call { name: String, args: Vec<Arg> },
    Ret(Arg),
}

impl std::fmt::Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Local(offset) => f.write_fmt(format_args!("Local({})", offset)),
            Arg::Literal(value) => f.write_fmt(format_args!("Literal({})", value)),
            Arg::DataOffset(offset) => f.write_fmt(format_args!("DataOffset({})", offset)),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::op::Op::StackAlloc(size) => {
                f.write_fmt(format_args!("    StackAlloc({})", size))
            }
            crate::op::Op::Invite { name } => f.write_fmt(format_args!("    Invite({})", name)),
            crate::op::Op::EternalAssign { offset, arg } => {
                f.write_fmt(format_args!("    EternalAssign({}, {})", offset, arg))
            }
            crate::op::Op::Label(name) => f.write_fmt(format_args!("{}:", name)),
            crate::op::Op::Call { name, args } => {
                f.write_fmt(format_args!("Call({}, {:?})", name, args))
            }
            crate::op::Op::Ret(arg) => f.write_fmt(format_args!("Ret({})", arg)),
        }
    }
}
