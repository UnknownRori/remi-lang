use crate::compiler::Compiler;

use super::{Codegen, CodegenError};

pub struct JavascriptCodegen;

impl JavascriptCodegen {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_prolog(&mut self, compiler: &Compiler, body: &mut Vec<String>) {
        body.push("// Remi Language v0.0".to_owned());
        let bytes: Vec<String> = compiler
            .eternal_value
            .iter()
            .map(|b| b.to_string())
            .collect();
        let data = format!("const DATA = new Uint8Array([{}]);", bytes.join(", "));
        body.push(data);
        body.push(
            "
function readString(offset) {
    let chars = [];
    while (DATA[offset] !== 0) {
        chars.push(String.fromCharCode(DATA[offset]));
        offset++;
    }
    return chars.join('');
}
"
            .to_owned(),
        );
        // TODO : generate this on the fly
        body.push("function remi () {".to_owned());
    }

    pub fn generate_epilog(&mut self, body: &mut Vec<String>) {
        body.push("}".to_owned());
        body.push("remi();".to_owned());
    }
}

impl Codegen for JavascriptCodegen {
    fn compile(
        &mut self,
        compiler: Compiler,
        stmt: Vec<crate::op::Op>,
    ) -> Result<String, super::CodegenError> {
        let mut code: Vec<String> = vec![];
        self.generate_prolog(&compiler, &mut code);
        for op in stmt {
            match op {
                crate::op::Op::StackAlloc(offset) => {
                    code.push(format!("    let _{};", offset));
                }
                crate::op::Op::Invite { name } => {
                    code.push(format!("import {{ {} }} from 'console'", name));
                }
                crate::op::Op::EternalAssign { offset, arg } => match arg {
                    crate::op::Arg::Local(local) => {
                        code.push(format!("    _{} = _{}", offset, local));
                    }
                    crate::op::Arg::Literal(value) => {
                        code.push(format!("    _{} = {}", offset, value.str()));
                    }
                    crate::op::Arg::DataOffset(offset) => {
                        code.push(format!("    _{} = readString({})", offset, offset))
                    }
                },
                crate::op::Op::Label(name) => {}
                crate::op::Op::Call { name, args } => {
                    let mapped = args.iter().map(|a| match a {
                        crate::op::Arg::Local(offset) => format!("_{}", offset),
                        crate::op::Arg::Literal(value) => format!("{}", value.str()),
                        crate::op::Arg::DataOffset(offset) => {
                            format!("readString({})", offset)
                        }
                    });
                    let mut arg = vec![];
                    for i in mapped {
                        arg.push(i);
                    }
                    let arg = arg.join(", ");
                    code.push(format!("    {}({});", name, arg));
                }
                crate::op::Op::Ret(arg) => {}
                other => Err(CodegenError::Unsupported { op: other })?,
            }
        }
        self.generate_epilog(&mut code);
        Ok(code.join("\n"))
    }
}
