use insta::assert_debug_snapshot;
use plrs::lexer::Lexer;

#[test]
fn not() {
    let input = r#"
            fn test() {
            let x = !true;
            }"#;

    let lexer = Lexer::new(input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}

#[test]
fn complex_math() {
    let input = r#"
            fn test() {
                let z = (2 * 2) / ((3 - 4) * -2);
            }
        "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}

#[test]
fn math() {
    let input = r#"
            fn test() {
                let x = 2 + 3 / 2 * 3 - 1;
            }
        "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}

#[test]
fn small_input() {
    let input = r#"
            const wow = 3;
            fn test() {}
        "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
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
}"#;

    let lexer = Lexer::new(input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}

#[test]
fn large_input() {
    let input = r#"
        const wow = 3;
        fn test(argv) {
            // this is a comment
            let a = "hello";
        }
        "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}
