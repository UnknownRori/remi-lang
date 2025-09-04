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

        body.push("; Remi v0.0 windows x86_64 assembly".to_string());
        body.push("format ms64 coff".to_string());
        body.push("section '.data' readable writeable".to_string());
        if bytes.len() > 0 {
            body.push(format!("    eternal: db {}", bytes.join(", ")));
        }
        body.push("section '.text' code readable executable".to_string());
        body.push("public main".to_string());
    }

    pub fn generate_epilog(&mut self, _body: &mut Vec<String>) {}
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
                op::Op::StackAlloc(count) => {
                    offset += 8 * count;
                    code.push(format!("    sub rsp, {}", 8 * count));
                    code.push(format!(""));
                }
                op::Op::Invite { name } => {
                    code.push(format!("extrn {}", name));
                }
                op::Op::EternalAssign { offset, arg } => {
                    code.push(format!("    ; Eternal Assign"));
                    match arg {
                        Arg::Local(id) => {
                            code.push(format!("    mov rcx, [rbp-{}]", (id + 1) * 8));
                            code.push(format!("    mov [rbp-{}], rcx", (offset + 1) * 8));
                        }
                        Arg::Literal(value) => code.push(format!(
                            "    mov qword [rbp-{}], {}",
                            (offset + 1) * 8,
                            value.str()
                        )),
                        Arg::DataOffset(data) => {
                            code.push(format!("    mov rcx, eternal+{}", data));
                            code.push(format!("    mov qword [rbp-{}], rcx", (offset + 1) * 8));
                        }
                    }

                    code.push(format!(""));
                }
                op::Op::Function(name) => {
                    offset = 0;
                    code.push(format!("{}:", name));
                    code.push(format!("    ; Prolog"));
                    code.push(format!("    push rbp"));
                    code.push(format!("    mov rbp, rsp"));
                }
                op::Op::Label(name) => {
                    code.push(format!("{}:", name));
                }
                op::Op::Call { name, args } => {
                    code.push(format!("    ; Calling"));
                    const REGISTER: [&str; 4] = ["rcx", "rdx", "r8", "r9"];
                    for (reg, arg) in REGISTER.iter().zip(args.iter()) {
                        match arg {
                            Arg::Local(id) => {
                                code.push(format!("    mov rax, [rbp-{}]", (id + 1) * 8));
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
                    let shadow = shadow_stack_space(offset);
                    code.push(format!("    sub rsp, {}", shadow - offset));
                    code.push(format!("    call {}", name));
                    code.push(format!("    add rsp, {}", shadow - offset));
                    code.push(format!(""));
                }
                op::Op::Ret(arg) => {
                    code.push(format!("    ; Epilog"));
                    if let Some(arg) = arg {
                        match arg {
                            Arg::Local(id) => {
                                code.push(format!("    mov rax, [rbp-{}]", (id + 1) * 8))
                            }
                            Arg::Literal(value) => {
                                code.push(format!("    mov rax, {}", value.str()))
                            }
                            Arg::DataOffset(offset) => {
                                code.push(format!("    mov rax, [eternal+{}]", offset))
                            }
                        }
                    }

                    code.push(format!("    add rsp, {}", offset));
                    code.push(format!("    pop rbp"));
                    code.push(format!("    ret"));
                }
                op::Op::UnaryNot { offset, arg } => {
                    code.push("    xor rbx, rbx".to_owned());
                    match arg {
                        Arg::Local(offset) => {
                            code.push(format!("    cmp rax, [rbp-{}]", (offset + 1) * 8))
                        }
                        Arg::Literal(value) => code.push(format!("    cmp rax, {}", value.str())),
                        Arg::DataOffset(offset) => code.push(format!("    cmp rax, [{}]", offset)),
                    }
                    code.push("    test al, al".to_owned());
                    code.push("    setz bl".to_owned());
                    code.push(format!("    mov [rbp-{}], rbx", (offset + 1) * 8));
                }
                op::Op::BinOp {
                    binop,
                    offset,
                    lhs,
                    rhs,
                } => {
                    code.push(format!("    ; Bin Op {}", binop));
                    match lhs {
                        Arg::Local(offset) => {
                            code.push(format!("    mov rax, [rbp-{}]", (offset + 1) * 8))
                        }
                        Arg::Literal(value) => code.push(format!("    mov rax, {}", value.str())),
                        Arg::DataOffset(offset) => {
                            code.push(format!("    mov rax, [eternal+{}]", offset))
                        }
                    }
                    match binop {
                        crate::ast::BinOp::Add => {
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    add rax, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    add rax, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    add rax, [{}]", offset))
                                }
                            }
                            code.push(format!("    mov [rbp-{}], rax", (offset + 1) * 8));
                        }
                        crate::ast::BinOp::Sub => {
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    sub rax, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    sub rax, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    sub rax, [{}]", offset))
                                }
                            }
                            code.push(format!("    mov [rbp-{}], rax", (offset + 1) * 8));
                        }
                        crate::ast::BinOp::Mul => {
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    imul rax, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    imul rax, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    imul rax, [{}]", offset))
                                }
                            }
                            code.push(format!("    mov [rbp-{}], rax", (offset + 1) * 8));
                        }
                        crate::ast::BinOp::Div => {
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    mov rbx, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    mov rbx, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    mov rbx, [{}]", offset))
                                }
                            }
                            code.push(format!("    xor rdx, rdx"));
                            code.push(format!("    div rbx"));
                            code.push(format!("    mov [rbp-{}], rax", (offset + 1) * 8));
                        }
                        crate::ast::BinOp::Equal => {
                            code.push("    xor rbx, rbx".to_owned());
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    cmp rax, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    cmp rax, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    cmp rax, [{}]", offset))
                                }
                            }
                            code.push("    setz bl".to_owned());
                            code.push(format!("    mov [rbp-{}], rbx", (offset + 1) * 8));
                        }
                        crate::ast::BinOp::Greater => {
                            code.push("    xor rbx, rbx".to_owned());
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    cmp rax, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    cmp rax, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    cmp rax, [{}]", offset))
                                }
                            }
                            code.push("    setg bl".to_owned());
                            code.push(format!("    mov [rbp-{}], rbx", (offset + 1) * 8));
                        }
                        crate::ast::BinOp::Less => {
                            code.push("    xor rbx, rbx".to_owned());
                            match rhs {
                                Arg::Local(offset) => {
                                    code.push(format!("    cmp rax, [rbp-{}]", (offset + 1) * 8))
                                }
                                Arg::Literal(value) => {
                                    code.push(format!("    cmp rax, {}", value.str()))
                                }
                                Arg::DataOffset(offset) => {
                                    code.push(format!("    cmp rax, [{}]", offset))
                                }
                            }
                            code.push("    setl bl".to_owned());
                            code.push(format!("    mov [rbp-{}], rbx", (offset + 1) * 8));
                        }
                    }

                    code.push(format!(""));
                }
                op::Op::Jmp { name } => code.push(format!("    jmp {}", name)),
                op::Op::JmpIfNot { name, arg } => {
                    code.push(format!("    ; Jump if not"));
                    match arg {
                        Arg::Local(offset) => {
                            code.push(format!("    mov rax, [rbp-{}]", (offset + 1) * 8))
                        }
                        Arg::Literal(value) => code.push(format!("    mov rax, {}", value.str())),
                        Arg::DataOffset(offset) => {
                            code.push(format!("    mov rax, [eternal+{}]", offset))
                        }
                    }
                    code.push(format!("    test rax, rax"));
                    code.push(format!("    jz {}", name));
                    code.push(format!(""));
                }
            }
        }
        Ok(code.join("\n"))
    }
}
fn shadow_stack_space(locals_size: usize) -> usize {
    let shadow_space = 32;
    let total = locals_size + shadow_space;

    (total + 15) & !15
}
