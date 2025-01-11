use insta::assert_debug_snapshot;
use plrs::lexer::Lexer;

#[test]
fn not() {
    let input = r#"
            fn test() {
            let x = !true;
            }"#;

    let lexer = Lexer::new(0, input);
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

    let lexer = Lexer::new(0, input);
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

    let lexer = Lexer::new(0, input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}

#[test]
fn small_input() {
    let input = r#"
            const wow = 3;
            fn test() {}
        "#;

    let lexer = Lexer::new(0, input);
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

    let lexer = Lexer::new(0, input);
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

    let lexer = Lexer::new(0, input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
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
        "#;

    let lexer = Lexer::new(0, input);
    let tokens = lexer.into_iter().collect::<Vec<_>>();

    assert_debug_snapshot!(tokens);
}
