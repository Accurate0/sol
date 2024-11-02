use insta::assert_compact_debug_snapshot;
use plrs::{
    compiler::Compiler,
    lexer::Lexer,
    parser::Parser,
    types,
    vm::{VMValue, VM},
};

#[test]
fn complex_math() {
    let input = r#"
        let z = (2 * 2) / ((3 - 4) * 2);
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let program = compiler.compile().unwrap();

    let vm = VM::new(program);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
}

#[test]
fn math() {
    let input = r#"
        let x = 1 + 2 / 3;
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let program = compiler.compile().unwrap();

    let vm = VM::new(program);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
}

#[test]
fn prefix() {
    let input = r#"
        let x = -1;
        let y = -(x + 3);
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let program = compiler.compile().unwrap();

    let vm = VM::new(program);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
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

    let program = compiler.compile().unwrap();

    let vm = VM::new(program);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
}

#[test]
fn native_function() {
    let input = r#"
        test_function();
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let program = compiler.compile().unwrap();

    let vm = VM::new(program).define_native_function("test_function".to_owned(), |_| None);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
}

#[test]
fn native_function_with_return_value() {
    let input = r#"
        let x = test();
        if x {
            print("pass");
        } else {
            print("fail");
        }
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let program = compiler.compile().unwrap();

    let vm = VM::new(program).define_native_function("test".to_owned(), |_| {
        Some(VMValue::Literal(std::borrow::Cow::Owned(
            types::Literal::Boolean(true),
        )))
    });
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
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

    let program = compiler.compile().unwrap();

    let vm = VM::new(program);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
}

#[test]
fn objects() {
    let input = r#"
let y = 3;

let another_object = {
    inner_value: 32,
};

let x = {
    test: 1,
    test2: "testing",
    test3: y,
    test4: another_object,
    test5: {
        test6: {
            test7: 1999
        }
    }
};

print(x);
print(x.test);
print(x.test2);
print(x.test3);
print(x.test4);
print(x.test4.inner_value);
print(x.test5);
print(x.test5.test6);
print(x.test5.test6.test7);
        "#
    .to_owned();

    let mut lexer = Lexer::new(&input);
    let mut parser = Parser::new(&mut lexer, &input);
    let compiler = Compiler::new(&mut parser);

    let program = compiler.compile().unwrap();

    let vm = VM::new(program);
    let register_state = vm.run_with_registers_returned();

    assert_compact_debug_snapshot!(register_state);
}
