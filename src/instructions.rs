use crate::{compiler::Value, vm::Register};

// #[repr(u8)]
// FIXME MAKE IT ACTUAL BYTECODE
#[derive(Debug)]
pub enum Instruction {
    GetVariable {
        dest: Register,
        src: String,
    },
    Load {
        dest: Register,
        value: Value,
    },
    SetVariable {
        src: Register,
        dest: String,
    },
    GetConstant {
        dest: Register,
        src: String,
    },
    Add {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    FunctionCall {
        name: String,
        args: Vec<Register>,
    },
}
