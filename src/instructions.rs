pub enum Instruction {}

impl Instruction {
    pub fn emit_bytes(&self, buffer: &mut Vec<u8>) {
        match *self {}
    }
}
