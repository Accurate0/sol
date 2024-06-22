use std::ops::Range;

pub type Register = u8;
pub type LiteralId = u16;
pub type FunctionId = u16;

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
    CallFunction {
        src: Register,
        args: Range<Register>,
    },
    LoadLiteral {
        dest: Register,
        src: LiteralId,
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
    Return,
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
