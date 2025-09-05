use crate::{
    compiler::Compiler,
    op::{Arg, Op},
};

use super::{Codegen, CodegenError, utils::align_mem};

pub struct LinuxX86_64;

const REGISTER: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl LinuxX86_64 {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_prolog(&mut self, compiler: &Compiler, body: &mut Vec<String>) {
        let bytes: Vec<String> = compiler
            .eternal_value
            .iter()
            .map(|b| format!("{}", b))
            .collect();

        body.push("; Remi v0.0 linux x86_64 assembly".to_string());
        body.push("format elf64".to_string());
        body.push("section '.data' writeable".to_string());
        if bytes.len() > 0 {
            body.push(format!("    eternal: db {}", bytes.join(", ")));
        }
        body.push("section '.text' executable".to_string());
        body.push("public main".to_string());
    }

    pub fn generate_epilog(&mut self, _body: &mut Vec<String>) {}
}

impl Codegen for LinuxX86_64 {
    fn compile(&mut self, compiler: Compiler, stmt: Vec<Op>) -> Result<String, CodegenError> {
        let mut code: Vec<String> = vec![];
        self.generate_prolog(&compiler, &mut code);
        let mut offset = 0;
        for op in stmt {
            match op {
                Op::StackAlloc(count) => {
                    offset = align_mem(offset + (count * 8));
                    code.push(format!("    sub rsp, {}", offset));
                    code.push(format!(""));
                }
                Op::Invite { name } => {
                    code.push(format!("extrn {}", name));
                }
                Op::EternalAssign { offset, arg } => {
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
                Op::Function(name) => {
                    offset = 0;
                    code.push(format!("{}:", name));
                    code.push(format!("    ; Prolog"));
                    code.push(format!("    push rbp"));
                    code.push(format!("    mov rbp, rsp"));
                }
                Op::Label(name) => {
                    code.push(format!("{}:", name));
                }
                Op::Call { name, args } => {
                    code.push(format!("    ; Calling"));
                    if args.len() > 4 {
                        return Err(crate::codegen::CodegenError::Unsupported {
                            op: Op::Call { name, args },
                        });
                    }
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
                    code.push(format!("    call {}", name));
                    code.push(format!(""));
                }
                Op::Ret(arg) => {
                    code.push(format!("    ; Epilog"));
                    if let Some(arg) = arg {
                        code.push(arg_to_reg(arg, "rax"));
                    }

                    if offset > 0 {
                        code.push(format!("    add rsp, {}", offset));
                    }
                    code.push(format!("    pop rbp"));
                    code.push(format!("    ret"));
                }
                Op::UnaryNot { offset, arg } => {
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
                Op::BinOp {
                    binop,
                    offset,
                    lhs,
                    rhs,
                } => {
                    code.push(format!("    ; Bin Op {}", binop));

                    code.push(arg_to_reg(lhs, "rax"));
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
                            code.push(arg_to_reg(rhs, "rbx"));
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
                Op::Jmp { name } => code.push(format!("    jmp {}", name)),
                Op::JmpIfNot { name, arg } => {
                    code.push(format!("    ; Jump if not"));
                    code.push(arg_to_reg(arg, "rax"));
                    code.push(format!("    test rax, rax"));
                    code.push(format!("    jz {}", name));
                    code.push(format!(""));
                }
            }
        }
        Ok(code.join("\n"))
    }
}

fn arg_to_reg(arg: Arg, reg: &str) -> String {
    match arg {
        Arg::Local(offset) => format!("    mov {}, [rbp-{}]", reg, (offset + 1) * 8),
        Arg::Literal(value) => format!("    mov {}, {}", reg, value.str()),
        Arg::DataOffset(offset) => format!("    mov {}, [eternal+{}]", reg, offset),
    }
}
