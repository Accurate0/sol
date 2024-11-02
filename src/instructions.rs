pub type Register = u8;
pub type LiteralId = u16;
pub type FunctionId = u16;
pub type JumpOffset = u16;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Copy {
        dest: Register,
        src: Register,
    },
    LoadFunction {
        dest: Register,
        src: FunctionId,
    },
    CallNativeFunction {
        src: Register,
        arg_count: u8,
        return_val: Register,
    },
    CallFunction {
        src: Register,
        arg_count: u8,
        return_val: Register,
    },
    AllocateObject {
        dest: Register,
    },
    SetObjectField {
        object: Register,
        field: Register,
        value: Register,
    },
    GetObjectField {
        object: Register,
        field: Register,
        return_val: Register,
    },
    LoadLiteral {
        dest: Register,
        src: LiteralId,
    },
    PrefixNot {
        dest: Register,
        rhs: Register,
    },
    PrefixSub {
        dest: Register,
        rhs: Register,
    },
    JumpIfFalse {
        src: Register,
        offset: JumpOffset,
    },
    Jump {
        offset: JumpOffset,
    },
    JumpReverse {
        offset: JumpOffset,
    },
    Add {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    Sub {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    Mul {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    Div {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    Equals {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    NotEquals {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    GreaterThan {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    GreaterThanOrEquals {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    LessThan {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    LessThanOrEquals {
        dest: Register,
        lhs: Register,
        rhs: Register,
    },
    Return {
        val: Register,
    },
    FunctionReturn,
}

#[cfg(test)]
mod test {
    use super::Instruction;
    use pretty_assertions::assert_eq;
    use std::mem::size_of;

    #[test]
    fn test_instruction_is_32_bits() {
        assert_eq!(size_of::<Instruction>(), 4);
    }
}
