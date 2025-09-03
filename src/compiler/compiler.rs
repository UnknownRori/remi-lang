use std::collections::HashMap;

use crate::{
    ast::{Expression, Statement},
    commons::Loc,
    i32,
    op::{Arg, Op},
    string,
    value::Value,
};

use super::{
    CompilerError,
    alloc::FrameAlloc,
    symbol::{FunctionStorage, FunctionSymbol},
};

pub struct Scope {
    frame_alloc: FrameAlloc,
    next_local: usize,
    locals: HashMap<String, usize>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            frame_alloc: FrameAlloc::default(),
            locals: HashMap::new(),
            next_local: 0,
        }
    }

    pub fn alloc_local(&mut self, name: &str) -> usize {
        let id = self.next_local;
        self.next_local += 1;
        self.locals.insert(name.to_owned(), id);
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
                    _ = self.parse_expression(scope, &mut ops, expr)?;
                }
                Statement::Invite { name } => {
                    self.spellcard.insert(
                        name,
                        FunctionSymbol {
                            args: vec![],
                            storage: FunctionStorage::External,
                            return_type: "void".to_string(),
                        },
                    );
                }
                Statement::Eternal { name, .. } => {
                    let offset = scope.alloc_local(&name);
                    scope.locals.insert(name, offset);
                    ops.push(Op::StackAlloc(offset));
                }
                Statement::Vow { .. } => todo!(),
                Statement::Assignment { name, value } => {
                    let offset = scope
                        .locals
                        .get(&name)
                        .ok_or(CompilerError::UndefinedVariable {
                            found: name,
                            loc: Loc::default(),
                        })?
                        .clone();
                    let arg = self.parse_expression(scope, &mut ops, value)?;

                    ops.push(Op::EternalAssign { offset, arg });
                }
                Statement::Foreseen { .. } => todo!(),
                Statement::Until { .. } => todo!(),
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
                    ops.push(Op::Label(name));
                    ops.append(&mut body);
                }
                Statement::Offer(expression) => {
                    let arg = self.parse_expression(scope, &mut ops, expression)?;
                    ops.push(Op::Ret(arg));
                }
            }
        }
        Ok(ops)
    }

    fn parse_expression(
        &mut self,
        scope: &mut Scope,
        ops: &mut Vec<Op>,
        expr: Expression,
    ) -> Result<Arg, CompilerError> {
        match expr {
            Expression::Literal(value) => match value {
                Value::I32(val) => Ok(Arg::Literal(i32!(val))),
                Value::String(val) => {
                    let mut bytes = val.clone().into_bytes();
                    let offset = self.eternal_value.len();
                    self.eternal_value.append(&mut bytes);
                    self.eternal_value.push(0);
                    self.eternal.insert(val, offset);
                    Ok(Arg::DataOffset(offset))
                }
            },
            Expression::Variable(offset) => {
                let offset = scope
                    .get_local(&offset)
                    .ok_or(CompilerError::UndefinedVariable {
                        found: offset,
                        loc: Loc::default(),
                    })?;
                Ok(Arg::Local(offset))
            }
            Expression::Binary { op, left, right } => todo!(),
            Expression::Call { function, args } => {
                let _spellcard =
                    self.spellcard
                        .get(&function)
                        .ok_or(CompilerError::UnknownFunction {
                            found: function.clone(),
                            loc: Loc::default(),
                        })?;

                let mut args_expr = vec![];
                for expr in args {
                    let arg = self.parse_expression(scope, ops, expr)?;
                    args_expr.push(arg);
                }

                ops.push(Op::Call {
                    name: function,
                    args: args_expr,
                });

                let offset = scope.alloc_local("__temp");
                Ok(Arg::Local(offset))
            }
        }
    }
}
