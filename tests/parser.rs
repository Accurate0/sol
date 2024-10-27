use insta::assert_debug_snapshot;
use plrs::{lexer::Lexer, parser::Parser};

#[test]
fn small_input() {
    let input = r#"
            const wow = 3;
            fn test() {}
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn larger_test() {
    let input = r#"
const wow = 3;

fn main(argv) {
    let x = 2;
    let y = true;
    print("test");
    print(1.3);


    print(x);
    print(2);

    test();
}

fn test(){
    if true {

    } else {
// comment
        print(2);
    }
}

fn new_function(arg1, arg2, arg3) {
{

    test ();
}
}"#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn complex_math() {
    let input = r#"
            fn test() {
                let z = (2 * 2) / ((3 - 4) * -2);
            }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn math() {
    let input = r#"
            fn test() {
                let x = 1 + 2 / 3;
            }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn large_input() {
    let input = r#"
        const wow = 3;
        fn test(argv) {
            // this is a comment
            let a = "hello";
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);

    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn function_call_return() {
    let input = r#"
        fn test() {
            let x = test2();
        }

        fn test2() {

        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

// ..? maybe illegal
#[test]
fn useless_expression() {
    let input = r#"
        fn test() {
            2 + 2.3;
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);

    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn function_call_with_addition() {
    let input = r#"
        fn test() {
            let x = test2() + 1;
        }

        fn test2() {

        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn variable_and_operation() {
    let input = r#"
        fn test() {
            let x = 1;
            let z = 2 + x;
            let y = x + 3;
            let r = x + z;
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn variable_mutation() {
    let input = r#"
        fn test() {
            let x = 1;
            x = 2 + x;
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn prefix() {
    let input = r#"
        fn test() {
            let x = -1;
            let y = -(x + 3);
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn prefix_boolean() {
    let input = r#"
        fn test() {
            let x = true;
            let y = !x;
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}

#[test]
fn if_else_conditions() {
    let input = r#"
if true {
    print("boolean constant if");
} else {
    print("boolean constant else");
}

if 2 == 0 {
    print("equality if");
} else {
    print("equality else");
}

if 2 >= 0 {
    print("greater than equal if");
} else {
    print("greater than equal else");
}

if 2 <= 0 {
    print("less than equal if");
} else {
    print("less than equal else");
}

if 2 > 0 {
    print("greater than if");
} else {
    print("greater than else");
}

if 2 < 0 {
    print("less than if");
} else {
    print("less than else");
}


if false {
    print("if");
} else if true {
    print("else if");
} else {
    print("else");
}
    "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let parser = Parser::new(&mut lexer, &input);
    let statements = parser.collect::<Vec<_>>();

    assert!(statements.iter().all(|s| s.is_ok()));

    let statements = statements
        .into_iter()
        .map(|s| s.unwrap())
        .collect::<Vec<_>>();

    assert_debug_snapshot!(statements);
}
