use crate::{ast::BinOp, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Local(usize),
    Literal(Value),
    DataOffset(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    StackAlloc(usize),
    Invite {
        name: String,
    },
    EternalAssign {
        offset: usize,
        arg: Arg,
    },
    UnaryNot {
        offset: usize,
        arg: Arg,
    },
    BinOp {
        binop: BinOp,
        offset: usize,
        lhs: Arg,
        rhs: Arg,
    },

    Function(String),
    Label(String),
    Call {
        name: String,
        args: Vec<Arg>,
    },
    Ret(Option<Arg>),
    Jmp {
        name: String,
    },
    JmpIfNot {
        name: String,
        arg: Arg,
    },
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
            crate::op::Op::BinOp {
                binop,
                offset,
                lhs,
                rhs,
            } => f.write_fmt(format_args!(
                "    BinOp {} {} {} {}",
                offset, binop, lhs, rhs
            )),
            crate::op::Op::Function(name) => f.write_fmt(format_args!("{}:", name)),
            crate::op::Op::Label(name) => f.write_fmt(format_args!("{}:", name)),
            crate::op::Op::UnaryNot { offset, arg } => {
                f.write_fmt(format_args!("UnaryNot({}, {:?})", offset, arg))
            }
            crate::op::Op::Call { name, args } => {
                f.write_fmt(format_args!("Call({}, {:?})", name, args))
            }
            crate::op::Op::Ret(arg) => match arg {
                Some(arg) => f.write_fmt(format_args!("Ret({})", arg)),
                None => f.write_fmt(format_args!("Ret(void)")),
            },
            crate::op::Op::Jmp { name } => f.write_fmt(format_args!("Jmp {}", name)),
            crate::op::Op::JmpIfNot { name, arg } => {
                f.write_fmt(format_args!("jnz {} {}", name, arg))
            }
        }
    }
}
