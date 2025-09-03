use crate::{compiler::Compiler, op::Arg};

use super::{Codegen, CodegenError};

pub struct IRCodegen;

impl Codegen for IRCodegen {
    fn compile(
        &mut self,
        compiler: Compiler,
        ops: Vec<crate::op::Op>,
    ) -> Result<String, CodegenError> {
        let mut body: Vec<String> = vec![];
        body.push("Remi IR v0.0\n".to_owned());
        body.push("Data:".to_owned());
        for (i, data) in compiler.eternal_value.chunks(8).enumerate() {
            let mut buf = format!("    {:#06x}: ", i * 8);
            for byte in data {
                buf.push_str(&format!("{:#04x} ", byte));
            }
            body.push(buf);
        }
        body.push("\nText:".to_owned());
        for op in ops {
            match op {
                crate::op::Op::StackAlloc(size) => {
                    body.push(format!("        StackAlloc({:#04x})", size))
                }
                crate::op::Op::Invite { name } => body.push(format!("        Invite({})", name)),
                crate::op::Op::EternalAssign { offset, arg } => body.push(format!(
                    "        EternalAssign({:#04x}, {})",
                    offset,
                    dump_args(&arg)
                )),
                crate::op::Op::UnaryNot { offset, arg } => {
                    body.push(format!("        UnaryNot({}, {})", offset, dump_args(&arg)))
                }
                crate::op::Op::BinOp {
                    binop,
                    offset,
                    lhs,
                    rhs,
                } => body.push(format!(
                    "        BinOp {:#04x} {} {} {}",
                    offset,
                    dump_args(&lhs),
                    binop,
                    dump_args(&rhs)
                )),
                crate::op::Op::Function(name) => body.push(format!("    {}():", name)),
                crate::op::Op::Label(name) => body.push(format!("    {}:", name)),
                crate::op::Op::Call { name, args } => {
                    let args = args.iter().map(dump_args).collect::<Vec<_>>().join(", ");
                    body.push(format!("        Call({}, [{}])", name, args))
                }
                crate::op::Op::Ret(arg) => match arg {
                    Some(arg) => body.push(format!("        Ret({})", dump_args(&arg))),
                    None => body.push(format!("        Ret(void)")),
                },
                crate::op::Op::Jmp { name } => body.push(format!("        Jmp({})", name)),
                crate::op::Op::JmpIfNot { name, arg } => {
                    body.push(format!("        Jne({}, {})", name, dump_args(&arg)))
                }
            }
        }
        Ok(body.join("\n"))
    }
}

fn dump_args(arg: &Arg) -> String {
    match arg {
        Arg::Local(offset) => format!("Local({:#04x})", offset),
        Arg::Literal(value) => value.str(),
        Arg::DataOffset(offset) => format!("DataOffset({:#04x})", offset),
    }
}
