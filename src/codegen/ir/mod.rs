use crate::compiler::Compiler;

use super::{Codegen, CodegenError};

pub struct IRCodegen;

impl Codegen for IRCodegen {
    fn compile(
        &mut self,
        compiler: Compiler,
        ops: Vec<crate::op::Op>,
    ) -> Result<String, CodegenError> {
        let mut body: Vec<String> = vec![];
        body.push("Data:".to_owned());
        for data in compiler.eternal_value.windows(8) {
            let mut buf = "    ".to_owned();
            for byte in data {
                buf.push_str(&format!("{:#02x} ", byte));
            }
            body.push(buf);
        }
        body.push("Text:".to_owned());
        for op in ops {
            match op {
                crate::op::Op::StackAlloc(size) => {
                    body.push(format!("        StackAlloc({})", size))
                }
                crate::op::Op::Invite { name } => body.push(format!("        Invite({})", name)),
                crate::op::Op::EternalAssign { offset, arg } => {
                    body.push(format!("        EternalAssign({}, {})", offset, arg))
                }
                crate::op::Op::Label(name) => body.push(format!("    {}:", name)),
                crate::op::Op::Call { name, args } => {
                    body.push(format!("        Call({}, {:?})", name, args))
                }
                crate::op::Op::Ret(arg) => body.push(format!("        Ret({})", arg)),
            }
        }
        Ok(body.join("\n"))
    }
}
