use crate::{
    ast::{BinOp, Expression, FunctionArgs, Statement},
    commons::Loc,
    i32,
    lexer::{Lexer, Token, TokenKind},
    string,
};

use super::parser::*;
use super::*;

#[test]
fn parse_basic_source_code() {
    let body = "
spellcard main() i32 {
    offer 69;
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![Statement::Offer(Expression::Literal(i32!(69)))],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_basic_source_code_with_eternal() {
    let body = "
spellcard main() i32 {
    eternal foo = 69;
    offer foo;
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![
            Statement::Eternal {
                name: "foo".to_owned(),
                annotation: None,
            },
            Statement::Assignment {
                name: "foo".to_owned(),
                value: Expression::Literal(i32!(69)),
            },
            Statement::Offer(Expression::Variable("foo".to_owned())),
        ],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_simple_bin_op() {
    let body = "
spellcard main() i32 {
    eternal foo = 35 + 34;
    offer foo;
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![
            Statement::Eternal {
                name: "foo".to_owned(),
                annotation: None,
            },
            Statement::Assignment {
                name: "foo".to_owned(),
                value: Expression::Binary {
                    op: BinOp::Add,
                    left: Box::new(Expression::Literal(i32!(35))),
                    right: Box::new(Expression::Literal(i32!(34))),
                },
            },
            Statement::Offer(Expression::Variable("foo".to_owned())),
        ],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_with_paren_bin_op() {
    let body = "
spellcard main() i32 {
    eternal foo = 2 * (12 + 4);
    offer foo;
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![
            Statement::Eternal {
                name: "foo".to_owned(),
                annotation: None,
            },
            Statement::Assignment {
                name: "foo".to_owned(),
                value: Expression::Binary {
                    op: BinOp::Mul,
                    left: Box::new(Expression::Literal(i32!(2))),
                    right: Box::new(Expression::Binary {
                        op: BinOp::Add,
                        left: Box::new(Expression::Literal(i32!(12))),
                        right: Box::new(Expression::Literal(i32!(4))),
                    }),
                },
            },
            Statement::Offer(Expression::Variable("foo".to_owned())),
        ],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_call_function_without_param() {
    let body = "
foo();
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Expression(Expression::Call {
        function: "foo".to_owned(),
        args: vec![],
    })];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_call_function_with_1_param() {
    let body = "
foo(1);
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Expression(Expression::Call {
        function: "foo".to_owned(),
        args: vec![Expression::Literal(i32!(1))],
    })];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_call_function_with_2_param() {
    let body = "
foo(1, 2);
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Expression(Expression::Call {
        function: "foo".to_owned(),
        args: vec![Expression::Literal(i32!(1)), Expression::Literal(i32!(2))],
    })];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_call_function_in_expression() {
    let body = "
foo = foo(1, 2);
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Assignment {
        name: "foo".to_string(),
        value: Expression::Call {
            function: "foo".to_owned(),
            args: vec![Expression::Literal(i32!(1)), Expression::Literal(i32!(2))],
        },
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_foreseen() {
    let body = "
foreseen foo {
    say(1);
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Foreseen {
        condition: Expression::Variable("foo".to_string()),
        then_branch: vec![Statement::Expression(Expression::Call {
            function: "say".to_string(),
            args: vec![Expression::Literal(i32!(1))],
        })],
        else_branch: None,
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_foreseen_in_spellcard() {
    let body = "
spellcard main() i32 {
    foreseen foo {
        say(1);
    }
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![Statement::Foreseen {
            condition: Expression::Variable("foo".to_string()),
            then_branch: vec![Statement::Expression(Expression::Call {
                function: "say".to_string(),
                args: vec![Expression::Literal(i32!(1))],
            })],
            else_branch: None,
        }],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_foreseen_with_otherwise() {
    let body = "
foreseen foo {
    say(1);
} otherwise {
    say(2);
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Foreseen {
        condition: Expression::Variable("foo".to_string()),
        then_branch: vec![Statement::Expression(Expression::Call {
            function: "say".to_string(),
            args: vec![Expression::Literal(i32!(1))],
        })],
        else_branch: Some(vec![Statement::Expression(Expression::Call {
            function: "say".to_string(),
            args: vec![Expression::Literal(i32!(2))],
        })]),
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_foreseen_with_otherwise_in_spellcard() {
    let body = "
spellcard main() i32 {
    foreseen foo {
        say(1);
    } otherwise {
        say(2);
    }
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![Statement::Foreseen {
            condition: Expression::Variable("foo".to_string()),
            then_branch: vec![Statement::Expression(Expression::Call {
                function: "say".to_string(),
                args: vec![Expression::Literal(i32!(1))],
            })],
            else_branch: Some(vec![Statement::Expression(Expression::Call {
                function: "say".to_string(),
                args: vec![Expression::Literal(i32!(2))],
            })]),
        }],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_until() {
    let body = "
until foo {
    say(1);
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::Until {
        condition: Expression::Variable("foo".to_string()),
        body: vec![Statement::Expression(Expression::Call {
            function: "say".to_string(),
            args: vec![Expression::Literal(i32!(1))],
        })],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_until_in_spellcard() {
    let body = "
spellcard main() i32 {
    until foo {
        say(1);
    }
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![],
        return_type: Some("i32".to_string()),
        body: vec![Statement::Until {
            condition: Expression::Variable("foo".to_string()),
            body: vec![Statement::Expression(Expression::Call {
                function: "say".to_string(),
                args: vec![Expression::Literal(i32!(1))],
            })],
        }],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_spellcard_param_1() {
    let body = "
spellcard main(foo: i32) i32 {
    offer foo;
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![FunctionArgs {
            name: "foo".to_string(),
            annotation: "i32".to_string(),
        }],
        return_type: Some("i32".to_string()),
        body: vec![Statement::Offer(Expression::Variable("foo".to_owned()))],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_spellcard_param_2() {
    let body = "
spellcard main(foo: i32, bar: i32) i32 {
    offer foo;
}
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![Statement::SpellCard {
        name: "main".to_owned(),
        args: vec![
            FunctionArgs {
                name: "foo".to_string(),
                annotation: "i32".to_string(),
            },
            FunctionArgs {
                name: "bar".to_string(),
                annotation: "i32".to_string(),
            },
        ],
        return_type: Some("i32".to_string()),
        body: vec![Statement::Offer(Expression::Variable("foo".to_owned()))],
    }];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}

#[test]
fn parse_vow() {
    let body = "
vow testing = 0;
        ";
    let chars = body.chars().collect::<Vec<_>>();

    let expected = vec![
        Statement::Vow {
            name: "testing".to_string(),
            annotation: None,
        },
        Statement::Assignment {
            name: "testing".to_string(),
            value: Expression::Literal(i32!(0)),
        },
    ];

    let lexer = Lexer::new(&chars);
    let mut parser = Parser::new(lexer);
    let ops = parser.parse().expect("Should parse correctly");
    for (i, expect) in expected.iter().enumerate() {
        assert_eq!(expect, ops.get(i).expect("Should have the same op length"));
    }
}
