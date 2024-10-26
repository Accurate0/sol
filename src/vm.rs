use crate::{
    ast,
    compiler::{CompiledProgram, Function},
    instructions::Instruction,
};
use std::borrow::Cow;
use thiserror::Error;

struct SavedCallFrame<'a> {
    pub ip: usize,
    pub code: &'a Vec<Instruction>,
    pub register_count: u8,
}

#[derive(Error, Debug)]
pub enum ExecutionError {}

#[derive(Default, Debug, Clone)]
pub enum RegisterValue<'a> {
    #[default]
    Empty,
    Literal(Cow<'a, ast::Literal>),
    Function(&'a Function),
}

pub struct VM {
    functions: Vec<Function>,
    global_code: Vec<Instruction>,
    global_register_count: u8,
    literals: Vec<ast::Literal>,
}

macro_rules! impl_binary_op {
    ($registers:expr, $dest: expr, $lhs:expr, $x:tt, $rhs:expr) => {
        match (&$registers[*$lhs as usize], &$registers[*$rhs as usize]) {
            (RegisterValue::Literal(lhs), RegisterValue::Literal(rhs)) => {
                let lhs = lhs.as_ref();
                let rhs = rhs.as_ref();

                match (lhs, rhs) {
                    (ast::Literal::Float(lhs), ast::Literal::Float(rhs)) => {
                        $registers[*$dest as usize] =
                            RegisterValue::Literal(Cow::Owned(ast::Literal::Float(lhs $x rhs)))
                    }
                    (ast::Literal::Float(lhs), ast::Literal::Integer(rhs)) => {
                        $registers[*$dest as usize] = RegisterValue::Literal(Cow::Owned(
                            ast::Literal::Float(*lhs $x *rhs as f64),
                        ))
                    }
                    (ast::Literal::Integer(lhs), ast::Literal::Float(rhs)) => {
                        $registers[*$dest as usize] = RegisterValue::Literal(Cow::Owned(
                            ast::Literal::Float(*lhs as f64 $x *rhs),
                        ))
                    }
                    (ast::Literal::Integer(lhs), ast::Literal::Integer(rhs)) => {
                        $registers[*$dest as usize] =
                            RegisterValue::Literal(Cow::Owned(ast::Literal::Integer(lhs $x rhs)))
                    }

                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    };
}

impl VM {
    pub fn new(compiled_program: CompiledProgram) -> Self {
        Self {
            functions: compiled_program.functions,
            global_code: compiled_program.global_code,
            global_register_count: compiled_program.global_register_count,
            literals: compiled_program.literals,
        }
    }

    fn print_registers(window: &[RegisterValue]) {
        for (i, item) in window.iter().enumerate() {
            match item {
                RegisterValue::Empty => {}
                RegisterValue::Literal(l) => tracing::info!("{i} {:?}", l),
                RegisterValue::Function(f) => tracing::info!("{i} {:?}", f.name),
            }
        }
    }

    pub fn run(&self) -> Result<(), ExecutionError> {
        let mut registers = Vec::<RegisterValue>::with_capacity(u8::MAX as usize);
        registers.resize_with(u8::MAX as usize, Default::default);

        let mut saved_call_frames = Vec::<SavedCallFrame>::new();
        let mut current_code = &self.global_code;
        let mut register_window = &mut registers[0..];
        let mut base_register = 0;

        let mut ip = 0;
        loop {
            if ip >= current_code.len() {
                break;
            }

            let current_instruction = &current_code[ip];
            tracing::info!("executing: {:?}", current_instruction);
            // tracing::info!("ip: {:?}", ip);
            // tracing::info!("code: {:?}", current_code);
            // tracing::info!("reg: {:?}", register_window);
            // tracing::info!("base_reg: {:?}", base_register);

            match current_instruction {
                Instruction::Return => {
                    if let Some(saved_call_frame) = saved_call_frames.pop() {
                        if let Some(current_call_frame) = saved_call_frames.last() {
                            base_register -= current_call_frame.register_count as usize;
                        } else {
                            base_register -= self.global_register_count as usize;
                        }

                        register_window = &mut registers[base_register..];

                        ip = saved_call_frame.ip + 1;
                        current_code = saved_call_frame.code;
                        continue;
                    };
                }
                Instruction::LoadFunction { dest, src } => {
                    let func = &self.functions[*src as usize];
                    register_window[*dest as usize] = RegisterValue::Function(func);
                }
                Instruction::CallFunction { src, args } => {
                    let func = &register_window[*src as usize];
                    let func = match func {
                        RegisterValue::Function(f) => f,
                        _ => unreachable!(),
                    };

                    // eprintln!("DEBUGPRINT[2]: vm.rs:123: arg_start={:#?}", arg_start);
                    // eprintln!("DEBUGPRINT[3]: vm.rs:124: arg_end={:#?}", arg_end);
                    // tracing::info!("func: {:?}", func);

                    let old_code = current_code;
                    let old_ip = ip;

                    current_code = &func.code;
                    ip = 0;

                    let old_base = base_register;
                    let register_count = &func.register_count;
                    if let Some(current_call_frame) = saved_call_frames.last() {
                        base_register += current_call_frame.register_count as usize;
                    } else {
                        base_register += self.global_register_count as usize;
                    }

                    let (old_function, new_function) = registers.split_at_mut(base_register);

                    // tracing::warn!("FUNCTION CALL: OLD");
                    // Self::print_registers(old_function);

                    let arg_start = old_base + args.start as usize;
                    let arg_end = old_base + args.end as usize;
                    let registers_to_copy = &old_function[arg_start..arg_end];

                    // tracing::warn!("FUNCTION CALL: COPY");
                    // Self::print_registers(registers_to_copy);

                    for (index, register) in registers_to_copy.iter().enumerate() {
                        new_function[index + 1] = register.clone();
                    }

                    // tracing::warn!("FUNCTION CALL: NEW");
                    // Self::print_registers(new_function);

                    register_window = &mut registers[base_register..];
                    saved_call_frames.push(SavedCallFrame {
                        ip: old_ip,
                        code: old_code,
                        register_count: *register_count,
                    });

                    // tracing::warn!("register: {:?}", register_window);
                    // tracing::warn!("register: {:?}", self.global_register_count);
                    // tracing::warn!("register: {:?}", register_count);

                    continue;
                }

                Instruction::LoadLiteral { dest, src } => {
                    let literal = &self.literals[*src as usize];
                    register_window[*dest as usize] =
                        RegisterValue::Literal(Cow::Borrowed(literal));
                }

                Instruction::Add { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, +, rhs)
                }

                Instruction::Sub { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, -, rhs)
                }

                Instruction::Mul { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, *, rhs)
                }

                Instruction::Div { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, /, rhs)
                }

                Instruction::Copy { dest, src } => {
                    register_window[*dest as usize] = register_window[*src as usize].clone()
                }
            }

            Self::print_registers(register_window);
            println!();

            ip += 1;
        }

        // dbg!(registers);

        Ok(())
    }
}
