use insta::assert_debug_snapshot;
use plrs::{compiler::Compiler, lexer::Lexer, parser::Parser};

#[test]
fn small_input() {
    let input = r#"
let x = 3;
let y = 4;
let z = x + y;

fn print() {}

fn test(a) {
    let y = 1.3 + a;
    {
        let z = y + 3;
    }

    let z = y + 2;
}

fn main() {
    let x = 1.3 + 3;
    {
	print("Hello");
	print(x);
	let y = test(4);
    }

    print(x);
}


main();
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}

#[test]
fn variable_mutation() {
    let input = r#"
let mut x = 3;
x = 2;
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}

#[test]
fn prefix() {
    let input = r#"
let x = -3;
let y = -(x + 3);
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}

#[test]
fn prefix_boolean() {
    let input = r#"
let x = true;
let y = !x;
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}

#[test]
fn if_statement_boolean() {
    let input = r#"
if false {
    print("boolean constant if");
} else if false {
    print("boolean constant else");
} else {
    print("final else");
}
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}

#[test]
fn if_statement() {
    let input = r#"
if true {
    print("pass");
} else {
    print("fail");
}

if 2 == 0 {
    print("fail");
} else {
    print("pass");
}

if 2 >= 0 {
    print("pass");
} else {
    print("fail");
}

if 2 <= 0 {
    print("fail");
} else {
    print("pass");
}

if 2 > 0 {
    print("pass");
} else {
    print("fail");
}

if 2 < 0 {
    print("fail");
} else {
    print("pass");
}


if false {
    print("fail");
} else if true {
    print("pass");
} else {
    print("fail");
}
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}

#[test]
fn nested_loop() {
    let input = r#"
let mut x = 0;
loop {
    let mut y = 0;
    loop {
        if y > 3 {
            print("exit loop");
            break;
        }

        y = y + 1;
        print(y);
    }

    if x > 3 {
        print("exit loop");
        break;
    }

    x = x + 1;
    print(x);
}
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let output = compiler.compile().unwrap();

    assert_debug_snapshot!(output);
}
