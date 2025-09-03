use crate::{
    compiler::Compiler,
    op::{self, Arg},
};

use super::Codegen;

pub struct WindowsX86_64;

impl WindowsX86_64 {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_prolog(&mut self, compiler: &Compiler, body: &mut Vec<String>) {
        let bytes: Vec<String> = compiler
            .eternal_value
            .iter()
            .map(|b| format!("{}", b))
            .collect();

        body.push("format ms64 coff".to_string());
        body.push("section '.data' readable writeable".to_string());
        body.push(format!("    eternal: db {}", bytes.join(", ")));
        body.push("section '.text' code readable executable".to_string());
        body.push("public main".to_string());
    }

    pub fn generate_epilog(&mut self, body: &mut Vec<String>) {}
}

impl Codegen for WindowsX86_64 {
    fn compile(
        &mut self,
        compiler: crate::compiler::Compiler,
        stmt: Vec<crate::op::Op>,
    ) -> Result<String, super::CodegenError> {
        let mut code: Vec<String> = vec![];
        self.generate_prolog(&compiler, &mut code);
        let mut offset = 0;
        for op in stmt {
            match op {
                op::Op::StackAlloc(_) => {
                    offset += 8;
                    code.push(format!("    sub rsp, {}", 8));
                    code.push(format!(""));
                }
                op::Op::Invite { name } => {
                    code.push(format!("extrn {}", name));
                }
                op::Op::EternalAssign { offset, arg } => {
                    match arg {
                        Arg::Local(id) => {
                            code.push(format!("    mov rcx, [rbp-{}]", id * 8));
                            code.push(format!("    mov [rbp-{}], rcx", offset * 8));
                        }
                        Arg::Literal(value) => code.push(format!(
                            "    mov qword [rbp-{}], {}",
                            offset * 8,
                            value.str()
                        )),
                        Arg::DataOffset(offset) => {
                            code.push(format!("    mov rcx, eternal+{}", offset));
                            code.push(format!("    mov qword [rbp-{}], rcx", offset));
                        }
                    }

                    code.push(format!(""));
                }
                op::Op::Label(name) => code.push(format!("{}:", name)),
                op::Op::Call { name, args } => {
                    const REGISTER: [&str; 4] = ["rcx", "rdx", "r8", "r9"];
                    for (reg, arg) in REGISTER.iter().zip(args.iter()) {
                        match arg {
                            Arg::Local(id) => {
                                code.push(format!("    mov rax, [rbp-{}]", id * 8));
                                code.push(format!("    mov {}, rax", reg));
                            }
                            Arg::Literal(value) => {
                                code.push(format!("    mov {}, {}", reg, value.str()))
                            }
                            Arg::DataOffset(offset) => {
                                code.push(format!("    mov rax, eternal+{}", offset));
                                code.push(format!("    mov {}, rax", reg));
                            }
                        }
                    }
                    code.push(format!("    call {}", name));
                    code.push(format!(""));
                }
                op::Op::Ret(arg) => {
                    match arg {
                        Arg::Local(id) => code.push(format!("    mov rcx, [rbp-{}]", id * 8)),
                        Arg::Literal(value) => code.push(format!("    mov rcx, {}", value.str())),
                        Arg::DataOffset(offset) => {
                            code.push(format!("    mov rcx, [eternal+{}]", offset))
                        }
                    }

                    code.push(format!("    add rsp, {}", offset));
                    code.push(format!("    ret"));
                    offset = 0;
                }
            }
        }
        Ok(code.join("\n"))
    }
}
