use std::fmt::Debug;

#[repr(u8)]
pub enum Instruction {
    Move = 0,     // A, B
    Call,         // A, B, C
    Jump,         // A
    LoadConstant, // A, B
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Move => write!(f, "Move"),
            Instruction::Call => write!(f, "Call"),
            Instruction::Jump => write!(f, "Jump"),
            Instruction::LoadConstant => write!(f, "LoadConstant"),
        }
    }
}

impl Instruction {
    fn emit_move(buffer: &mut Vec<u32>, a: u8, b: u8) {
        let ins = (Instruction::Move as u32) << 24 | (a as u32) << 16 | (b as u32);
        buffer.push(ins);
    }

    fn emit_call(buffer: &mut Vec<u32>, a: u8, b: u8, c: u8) {
        let ins =
            (Instruction::Call as u32) << 24 | (a as u32) << 16 | (b as u32) << 8 | (c as u32);
        buffer.push(ins);
    }

    fn emit_jump(buffer: &mut Vec<u32>, a: u16) {
        let ins = (Instruction::Jump as u32) << 24 | (a as u32) << 16;
        buffer.push(ins);
    }

    fn emit_load_constant(buffer: &mut Vec<u32>, a: u8, b: u8) {
        let ins = (Instruction::LoadConstant as u32) << 24 | (a as u32) << 16 | (b as u32);
        buffer.push(ins);
    }
}
