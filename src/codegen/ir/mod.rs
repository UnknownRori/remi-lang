use super::{Codegen, CodegenError};

pub struct IRCodegen;

impl Codegen for IRCodegen {
    fn compile(&mut self, ops: Vec<crate::op::Op>) -> Result<String, CodegenError> {
        let mut body = vec![];
        for op in ops {
            match op {
                crate::op::Op::StackAlloc(size) => body.push(format!("    StackAlloc({})", size)),
                crate::op::Op::EternalAssign { offset, arg } => {
                    body.push(format!("    EternalAssign({}, {})", offset, arg))
                }
                crate::op::Op::Label(name) => body.push(format!("{}:", name)),
                crate::op::Op::Ret(arg) => body.push(format!("    Ret({})", arg)),
            }
        }
        Ok(body.join("\n"))
    }
}
