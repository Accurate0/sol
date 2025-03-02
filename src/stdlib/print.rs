use crate::vm::VMValue;

pub fn print(args: Vec<VMValue>) -> Option<VMValue> {
    for arg in args {
        match arg {
            VMValue::Empty => print!("<empty>"),
            VMValue::Literal(literal) => print!("{}", literal.as_ref()),
            VMValue::Function(f) => print!("{}", f),
            VMValue::Object(object) => print!("{}", object.borrow()),
            VMValue::Array(array) => print!("{}", array.borrow()),
        }
    }

    println!();

    None
}
