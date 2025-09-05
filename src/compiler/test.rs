use crate::i32;
use crate::{
    lexer::Lexer,
    op::{Arg, Op},
    parser::parser::Parser,
};

use super::Compiler;

fn setup(body: &str) -> (Vec<Op>, Compiler) {
    let chars = body.chars().collect::<Vec<_>>();

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse().expect("Should parse correctly");
    let mut compiler = Compiler::new();
    let ops = compiler.compile(ast).expect("Should compile ast correctly");

    (ops, compiler)
}

#[test]
pub fn compile_simple_source_code() {
    let body = "
spellcard main() i32 {
    offer 69;
}";

    let expected: Vec<Op> = vec![
        Op::Function("main".to_owned()),
        Op::Ret(Some(Arg::Literal(i32!(69)))),
    ];

    let (ops, _) = setup(body);
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
pub fn compile_source_code_with_eternal_1() {
    let body = "
spellcard main() i32 {
    eternal a = 69;
    offer a;
}";

    let expected: Vec<Op> = vec![
        Op::Function("main".to_owned()),
        Op::StackAlloc(1),
        Op::EternalAssign {
            arg: Arg::Literal(i32!(69)),
            offset: 0,
        },
        Op::Ret(Some(Arg::Local(0))),
    ];

    let (ops, _) = setup(body);
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
pub fn compile_source_code_with_until() {
    let body = "
spellcard main() i32 {
    eternal a = 69;
    until a > 0 {
        a = a - 1;
    }
    offer a;
}";

    let expected: Vec<Op> = vec![
        Op::Function("main".to_owned()),
        Op::StackAlloc(2),
        Op::EternalAssign {
            arg: Arg::Literal(i32!(69)),
            offset: 0,
        },
        Op::Label(".L0".to_string()),
        Op::BinOp {
            binop: crate::ast::BinOp::Greater,
            offset: 1,
            lhs: Arg::Local(0),
            rhs: Arg::Literal(i32!(0)),
        },
        Op::JmpIfNot {
            name: ".L1".to_string(),
            arg: Arg::Local(1),
        },
        Op::BinOp {
            binop: crate::ast::BinOp::Sub,
            offset: 1,
            lhs: Arg::Local(0),
            rhs: Arg::Literal(i32!(1)),
        },
        Op::EternalAssign {
            arg: Arg::Local(1),
            offset: 0,
        },
        Op::Jmp {
            name: ".L0".to_string(),
        },
        Op::Label(".L1".to_string()),
        Op::Ret(Some(Arg::Local(0))),
    ];

    let (ops, _) = setup(body);
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
pub fn compile_source_code_with_foreseen() {
    let body = "
spellcard main() i32 {
    eternal a = 69;
    foreseen a > 0 {
        a = a - 1;
    }
    offer a;
}";

    let expected: Vec<Op> = vec![
        Op::Function("main".to_owned()),
        Op::StackAlloc(2),
        Op::EternalAssign {
            arg: Arg::Literal(i32!(69)),
            offset: 0,
        },
        Op::BinOp {
            binop: crate::ast::BinOp::Greater,
            offset: 1,
            lhs: Arg::Local(0),
            rhs: Arg::Literal(i32!(0)),
        },
        Op::JmpIfNot {
            name: ".L0".to_string(),
            arg: Arg::Local(1),
        },
        Op::BinOp {
            binop: crate::ast::BinOp::Sub,
            offset: 1,
            lhs: Arg::Local(0),
            rhs: Arg::Literal(i32!(1)),
        },
        Op::EternalAssign {
            arg: Arg::Local(1),
            offset: 0,
        },
        Op::Label(".L0".to_string()),
        Op::Ret(Some(Arg::Local(0))),
    ];

    let (ops, _) = setup(body);
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}
