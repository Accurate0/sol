use std::{
    cell::RefCell,
    ops::{Index, IndexMut, Range},
};

use crate::instructions::Register;

use super::VMValue;

pub struct Registers<'a> {
    registers: Vec<VMValue<'a>>,
    base_register: RefCell<usize>,
}

impl<'a> Registers<'a> {
    pub fn new() -> Self {
        let mut s = Self {
            registers: Vec::with_capacity(u8::MAX as usize * 4),
            base_register: RefCell::new(0),
        };

        s.registers.resize_with(u8::MAX as usize, Default::default);

        s
    }

    pub fn regs_mut(&mut self) -> &mut Vec<VMValue<'a>> {
        &mut self.registers
    }

    pub fn regs(&self) -> &Vec<VMValue<'a>> {
        &self.registers
    }

    pub fn base_register(&self) -> usize {
        *self.base_register.borrow_mut()
    }

    pub fn update_base_register(&self, new_base: usize) {
        self.base_register.replace(new_base);
    }
}

impl<'a> Default for Registers<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Registers<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.registers)
    }
}

impl<'a> Index<Register> for Registers<'a> {
    type Output = VMValue<'a>;

    fn index(&self, index: Register) -> &Self::Output {
        let base_register = self.base_register();
        &self.registers[base_register + index as usize]
    }
}

impl<'a> Index<Range<Register>> for Registers<'a> {
    type Output = [VMValue<'a>];

    fn index(&self, index: Range<Register>) -> &Self::Output {
        let base_register = self.base_register();
        &self.registers
            [(base_register + index.start as usize)..(base_register + index.end as usize)]
    }
}

impl<'a> IndexMut<Register> for Registers<'a> {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        let base_register = self.base_register();
        &mut self.registers[base_register + index as usize]
    }
}
