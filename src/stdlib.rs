use crate::vm::{self, RegisterValue};
use phf::phf_map;

pub type NativeFunctionType = fn(Vec<RegisterValue>);

pub static STANDARD_LIBRARY: phf::Map<&'static str, NativeFunctionType> = phf_map! {
    "print" => print,
};

fn print(args: Vec<RegisterValue>) {
    for arg in args {
        match arg {
            vm::RegisterValue::Empty => print!("<empty>"),
            vm::RegisterValue::Literal(literal) => print!("{}", literal.as_ref()),
            vm::RegisterValue::Function(f) => {
                print!("<function: {} - code len: {}>", f.name, f.code.len())
            }
        }
    }

    println!();
}
