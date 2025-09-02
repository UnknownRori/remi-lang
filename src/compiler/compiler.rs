use std::collections::HashMap;

use crate::{
    ast::{Expression, Statement},
    commons::Loc,
    op::{Arg, Op},
};

use super::{CompilerError, alloc::FrameAlloc, symbol::FunctionSymbol};

pub struct Compiler {
    ops: Vec<Op>,
    frame_alloc: FrameAlloc,

    spellcard: HashMap<String, FunctionSymbol>,
    locals: HashMap<String, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            ops: vec![],
            frame_alloc: FrameAlloc::default(),

            spellcard: HashMap::new(),
            locals: HashMap::new(),
        }
    }

    pub fn compile(&mut self, ast: Vec<Statement>) -> Result<Vec<Op>, CompilerError> {
        self.compile_statement(ast)
    }

    fn emit(&mut self, op: Op) {
        self.ops.push(op);
    }

    fn compile_statement(&mut self, stmts: Vec<Statement>) -> Result<Vec<Op>, CompilerError> {
        let mut ops = vec![];
        for stmt in stmts {
            let mut op = match stmt {
                Statement::Eternal { name, .. } => {
                    self.locals.insert(name.clone(), self.frame_alloc.alloc(8));
                    vec![Op::StackAlloc(8)]
                }
                Statement::Vow { .. } => todo!(),
                Statement::Assignment { name, value } => {
                    let arg = self.parse_expression(value)?;
                    match self.locals.get(&name) {
                        Some(offset) => vec![Op::EternalAssign {
                            offset: *offset,
                            arg,
                        }],
                        None => todo!(),
                    }
                }
                Statement::Foreseen { .. } => todo!(),
                Statement::Until { .. } => todo!(),
                Statement::SpellCard {
                    name,
                    return_type,
                    body,
                    ..
                } => {
                    self.spellcard.insert(
                        name.clone(),
                        FunctionSymbol {
                            args: vec![],
                            return_type: return_type.unwrap_or("void".to_string()),
                        },
                    );
                    let mut ops = vec![Op::Label(name)];
                    let mut body = self.compile_statement(body)?;
                    ops.append(&mut body);
                    ops
                }
                Statement::Offer(expression) => {
                    let arg = self.parse_expression(expression)?;
                    vec![Op::Ret(arg)]
                }
                _ => unreachable!("Hit something"),
            };

            ops.append(&mut op);
        }
        Ok(ops)
    }

    fn parse_expression(&mut self, expr: Expression) -> Result<Arg, CompilerError> {
        match expr {
            Expression::Literal(value) => Ok(Arg::Literal(value.clone())),
            Expression::Variable(str) => match self.locals.get(&str) {
                Some(offset) => Ok(Arg::Local(*offset)),
                None => Err(CompilerError::UndefinedVariable {
                    found: str,
                    loc: Loc::default(),
                }),
            },
            Expression::Binary { .. } => todo!(),
            Expression::Call { .. } => todo!(),
        }
    }
}
