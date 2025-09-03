use std::collections::HashMap;

use crate::{
    ast::{Expression, Statement},
    commons::Loc,
    i32,
    op::{Arg, Op},
    value::Value,
};

use super::{
    CompilerError,
    symbol::{FunctionStorage, FunctionSymbol},
};

pub struct Scope {
    next_local: usize,
    locals: HashMap<String, usize>,
    label_count: usize,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            next_local: 0,
            label_count: 0,
        }
    }

    pub fn alloc_local(&mut self, name: &str) -> usize {
        let id = self.next_local;
        self.next_local += 1;
        self.locals.insert(name.to_owned(), id);
        id
    }

    pub fn alloc_label(&mut self) -> usize {
        let id = self.label_count;
        self.label_count += 1;
        id
    }

    pub fn get_local(&self, name: &str) -> Option<usize> {
        self.locals.get(name).copied()
    }
}

pub struct Compiler {
    pub eternal: HashMap<String, usize>,
    pub eternal_value: Vec<u8>,
    pub spellcard: HashMap<String, FunctionSymbol>,
    pub spellcard_scope: HashMap<String, Scope>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            eternal: HashMap::new(),
            eternal_value: vec![],
            spellcard: HashMap::new(),
            spellcard_scope: HashMap::new(),
        }
    }

    pub fn compile(&mut self, ast: Vec<Statement>) -> Result<Vec<Op>, CompilerError> {
        let mut scope = Scope::new();
        let op = self.compile_statement(&mut scope, ast)?;
        self.spellcard_scope.insert("__global".to_owned(), scope);
        Ok(op)
    }

    fn compile_statement(
        &mut self,
        scope: &mut Scope,
        ast: Vec<Statement>,
    ) -> Result<Vec<Op>, CompilerError> {
        let mut ops = vec![];
        for i in ast {
            match i {
                Statement::Expression(expr) => {
                    let (_, mut op) = self.parse_expression(scope, expr)?;
                    ops.append(&mut op);
                }
                Statement::Invite { name } => {
                    self.spellcard.insert(
                        name.to_owned(),
                        FunctionSymbol {
                            args: vec![],
                            storage: FunctionStorage::External,
                            return_type: "void".to_string(),
                        },
                    );
                    ops.push(Op::Invite { name });
                }
                Statement::Eternal { name, .. } => {
                    let offset = scope.alloc_local(&name);
                    scope.locals.insert(name, offset);
                }
                Statement::Vow { name, .. } => {
                    let offset = scope.alloc_local(&name);
                    scope.locals.insert(name, offset);
                }
                Statement::Assignment { name, value } => {
                    let offset = scope
                        .locals
                        .get(&name)
                        .ok_or(CompilerError::UndefinedVariable {
                            found: name,
                            loc: Loc::default(),
                        })?
                        .clone();
                    let (arg, mut op) = self.parse_expression(scope, value)?;

                    ops.append(&mut op);
                    ops.push(Op::EternalAssign { offset, arg });
                }
                Statement::Foreseen {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let id = scope.alloc_label();
                    let otherwise = format!(".L{}", id);

                    let id = scope.alloc_label();
                    let end = format!(".L{}", id);

                    let (arg, mut op_condition) = self.parse_expression(scope, condition)?;
                    ops.append(&mut op_condition);

                    let mut then_body = self.compile_statement(scope, then_branch)?;
                    match else_branch {
                        Some(body) => {
                            let mut else_body = self.compile_statement(scope, body)?;

                            ops.push(Op::JmpIfNot {
                                name: otherwise.clone(),
                                arg,
                            });
                            ops.append(&mut then_body);
                            ops.push(Op::Jmp {
                                name: end.to_owned(),
                            });
                            ops.push(Op::Label(otherwise));
                            ops.append(&mut else_body);
                        }
                        None => {
                            ops.push(Op::JmpIfNot {
                                name: end.clone(),
                                arg,
                            });
                            ops.append(&mut then_body);
                        }
                    }
                    ops.push(Op::Label(end));
                }
                Statement::Until { condition, body } => {
                    let id = scope.alloc_label();
                    let start = format!(".L{}", id);

                    let id = scope.alloc_label();
                    let end = format!(".L{}", id);

                    let (arg, mut op) = self.parse_expression(scope, condition)?;
                    let mut body = self.compile_statement(scope, body)?;

                    ops.push(Op::Label(start.clone()));
                    ops.append(&mut op);
                    ops.push(Op::JmpIfNot {
                        name: end.clone(),
                        arg,
                    });
                    ops.append(&mut body);
                    ops.push(Op::Jmp { name: start });
                    ops.push(Op::Label(end));
                }
                Statement::SpellCard { name, body, .. } => {
                    let mut scope = Scope::new();
                    let mut body = self.compile_statement(&mut scope, body)?;
                    self.spellcard.insert(
                        name.to_owned(),
                        FunctionSymbol {
                            args: vec![],
                            return_type: "void".to_owned(),
                            storage: FunctionStorage::Internal,
                        },
                    );
                    ops.push(Op::Function(name));
                    ops.push(Op::StackAlloc(scope.next_local));
                    ops.append(&mut body);
                }
                Statement::Offer(expression) => match expression {
                    Some(expression) => {
                        let (arg, mut op) = self.parse_expression(scope, expression)?;
                        ops.append(&mut op);
                        ops.push(Op::Ret(Some(arg)));
                    }
                    None => {
                        ops.push(Op::Ret(None));
                    }
                },
            }
        }
        Ok(ops)
    }

    fn parse_expression(
        &mut self,
        scope: &mut Scope,
        expr: Expression,
    ) -> Result<(Arg, Vec<Op>), CompilerError> {
        match expr {
            Expression::Literal(value) => match value {
                Value::I32(val) => Ok((Arg::Literal(i32!(val)), vec![])),
                Value::String(val) => {
                    let mut bytes = val.clone().into_bytes();
                    let offset = self.eternal_value.len();
                    self.eternal_value.append(&mut bytes);
                    self.eternal_value.push(0);
                    self.eternal.insert(val, offset);
                    Ok((Arg::DataOffset(offset), vec![]))
                }
            },
            Expression::Variable(offset) => {
                let offset = scope
                    .get_local(&offset)
                    .ok_or(CompilerError::UndefinedVariable {
                        found: offset,
                        loc: Loc::default(),
                    })?;
                Ok((Arg::Local(offset), vec![]))
            }
            Expression::Unary { op, arg } => {
                let mut opsbin = vec![];
                let (lhs, mut opl) = self.parse_expression(scope, *arg)?;
                let offset = scope.alloc_local("__temp");
                opsbin.append(&mut opl);
                match op {
                    crate::ast::UnaryOp::Not => {
                        opsbin.push(Op::UnaryNot { offset, arg: lhs });
                    }
                }
                Ok((Arg::Local(offset), opsbin))
            }
            Expression::Binary { op, left, right } => {
                let mut opsbin = vec![];
                let (lhs, mut opl) = self.parse_expression(scope, *left)?;
                let (rhs, mut opr) = self.parse_expression(scope, *right)?;
                let offset = scope.alloc_local("__temp");

                opsbin.append(&mut opl);
                opsbin.append(&mut opr);
                opsbin.push(Op::BinOp {
                    binop: op,
                    offset,
                    lhs,
                    rhs,
                });

                Ok((Arg::Local(offset), opsbin))
            }
            Expression::Call { function, args } => {
                let _spellcard =
                    self.spellcard
                        .get(&function)
                        .ok_or(CompilerError::UnknownFunction {
                            found: function.clone(),
                            loc: Loc::default(),
                        })?;

                let mut ops = vec![];
                let mut args_expr = vec![];
                for expr in args {
                    let (arg, mut op) = self.parse_expression(scope, expr)?;
                    ops.append(&mut op);
                    args_expr.push(arg);
                }

                ops.push(Op::Call {
                    name: function,
                    args: args_expr,
                });

                let offset = scope.alloc_local("__temp");
                Ok((Arg::Local(offset), ops))
            }
        }
    }
}
