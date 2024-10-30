use crate::vm::RegisterValue;

pub fn print(args: Vec<RegisterValue>) -> Option<RegisterValue> {
    for arg in args {
        match arg {
            RegisterValue::Empty => print!("<empty>"),
            RegisterValue::Literal(literal) => print!("{}", literal.as_ref()),
            RegisterValue::Function(f) => {
                print!("<function: {} - code len: {}>", f.name, f.code.len())
            }
        }
    }

    println!();

    None
}
