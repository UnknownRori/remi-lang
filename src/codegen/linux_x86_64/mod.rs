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
        // TODO : Make function visibility
        for i in compiler.spellcard.iter() {
            match i.1.storage {
                crate::compiler::FunctionStorage::Internal => body.push(format!("public {}", &i.0)),
                _ => {}
            }
        }
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
                Op::Call { result, name, args } => {
                    code.push(format!("    ; Calling"));
                    let mut stack_size = 0;
                    if args.len() > REGISTER.len() {
                        let stack_count = args.len() - REGISTER.len();
                        stack_size = align_mem(stack_count * 8);
                        code.push(format!("    sub rsp, {}", stack_size));
                        for (i, arg) in args.iter().skip(REGISTER.len()).enumerate() {
                            match arg {
                                Arg::Local(offset) => code.push(format!(
                                    "    mov qword [rsp+{}], [rbp-{}]",
                                    i * 8,
                                    (offset + 1) * 8
                                )),
                                Arg::Literal(value) => code.push(format!(
                                    "    mov qword [rsp+{}], {}",
                                    i * 8,
                                    value.str()
                                )),
                                Arg::DataOffset(offset) => code.push(format!(
                                    "    mov qword [rsp+{}], eternal+{}",
                                    i * 8,
                                    offset
                                )),
                            }
                        }
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
                    code.push(format!("    mov [rbp-{}], rax", (result + 1) * 8));
                    if args.len() > 5 {
                        code.push(format!("    add rsp, {}", stack_size));
                    }
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
                Op::ParamAssign { offset, ref arg } => {
                    if let Some(reg) = REGISTER.get(offset) {
                        match arg {
                            Arg::Local(offset) => {
                                code.push(format!("    mov [rbp-{}], {}", (offset + 1) * 8, reg))
                            }
                            _ => {
                                return Err(super::CodegenError::InvalidOperation {
                                    message: format!(
                                        "Function parameter can only assign on local variable"
                                    ),
                                });
                            }
                        }
                    } else {
                        match arg {
                            Arg::Local(local) => {
                                code.push(format!(
                                    "    mov rax, [rbp+16+{}]",
                                    (offset - REGISTER.len()) * 8
                                ));
                                code.push(format!("    mov qword [rbp-{}], rax", (local + 1) * 8,));
                            }
                            _ => {
                                return Err(super::CodegenError::InvalidOperation {
                                    message: format!(
                                        "Function parameter can only assign on local variable"
                                    ),
                                });
                            }
                        }
                    }
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
