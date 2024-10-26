use insta::assert_compact_debug_snapshot;
use plrs::{compiler::Compiler, lexer::Lexer, parser::Parser, vm::VM};

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
