use crate::{
    ast::{self, Literal},
    compiler::{CompiledProgram, Function},
    impl_binary_comparator, impl_binary_op,
    instructions::Instruction,
    stdlib::{NativeFunctionType, STANDARD_LIBRARY},
};
use std::{borrow::Cow, collections::HashMap};
use thiserror::Error;

mod register;
pub use register::*;

struct SavedCallFrame<'a> {
    // FIXME: we could store the register array with each callframe
    // right now each function is allowed 256 registers by the compiler
    // but the VM only has 256 total, a deep call stack will easily go
    // beyond this number, we'd need more memory but could allocate
    // the right number of registers for each function
    // or make the window resizable... abstract the window away and resize on access
    pub ip: usize,
    pub code: &'a Vec<Instruction>,
    pub register_count: u8,
    pub function_return_value: u8,
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("{cause}")]
    InvalidOperation { cause: String },
}

pub struct VM {
    functions: Vec<Function>,
    native_functions: HashMap<String, NativeFunctionType>,
    global_code: Vec<Instruction>,
    global_register_count: u8,
    literals: Vec<ast::Literal>,
}

impl VM {
    pub fn new(compiled_program: CompiledProgram) -> Self {
        Self {
            functions: compiled_program.functions,
            native_functions: Default::default(),
            global_code: compiled_program.global_code,
            global_register_count: compiled_program.global_register_count,
            literals: compiled_program.literals,
        }
    }

    #[allow(unused)]
    pub fn define_native_function(mut self, name: String, function: NativeFunctionType) -> Self {
        self.native_functions.insert(name, function);

        self
    }

    fn print_registers(window: &[RegisterValue]) {
        for (i, item) in window.iter().enumerate() {
            match item {
                RegisterValue::Empty => {}
                RegisterValue::Literal(l) => tracing::debug!("{i} {:?}", l),
                RegisterValue::Function(f) => tracing::debug!("{i} {:?}", f.name),
            }
        }

        tracing::debug!("");
    }

    pub fn run_with_registers_returned(&self) -> Result<Vec<RegisterValue>, ExecutionError> {
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
            tracing::debug!("executing: {:?}", current_instruction);
            // tracing::info!("ip: {:?}", ip);
            // tracing::info!("code: {:?}", current_code);
            // tracing::info!("reg: {:?}", register_window);
            // tracing::info!("base_reg: {:?}", base_register);

            match current_instruction {
                Instruction::FunctionReturn => {
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
                Instruction::Return { val } => {
                    if let Some(saved_call_frame) = saved_call_frames.pop() {
                        if let Some(current_call_frame) = saved_call_frames.last() {
                            base_register -= current_call_frame.register_count as usize;
                        } else {
                            base_register -= self.global_register_count as usize;
                        }

                        let register_to_copy_to = saved_call_frame.function_return_value;
                        let register_to_copy_from = *val;

                        let from = register_window[register_to_copy_from as usize].clone();

                        register_window = &mut registers[base_register..];

                        register_window[register_to_copy_to as usize] = from;

                        ip = saved_call_frame.ip + 1;
                        current_code = saved_call_frame.code;
                        continue;
                    };
                }
                Instruction::LoadFunction { dest, src } => {
                    let func = &self.functions[*src as usize];
                    register_window[*dest as usize] = RegisterValue::Function(func);

                    ip += 1;
                }
                Instruction::CallNativeFunction {
                    src,
                    arg_count,
                    return_val,
                } => {
                    let register = &register_window[*src as usize];
                    let function_name = match register {
                        RegisterValue::Literal(cow) => match cow.as_ref() {
                            Literal::String(s) => s,
                            _ => {
                                return Err(ExecutionError::InvalidOperation {
                                    cause: "native function name must be string value".to_owned(),
                                })
                            }
                        },

                        _ => {
                            return Err(ExecutionError::InvalidOperation {
                                cause: "native function name must be a literal".to_owned(),
                            })
                        }
                    };

                    // TODO: could be slow to check native function list every
                    let native_function = self
                        .native_functions
                        .get(function_name)
                        .or_else(|| STANDARD_LIBRARY.get(function_name));

                    if native_function.is_none() {
                        return Err(ExecutionError::InvalidOperation {
                            cause: format!("no function matching name '{}' found", function_name)
                                .to_owned(),
                        });
                    }

                    let native_function = native_function.unwrap();

                    let arg_start = *src as usize - *arg_count as usize;
                    let arg_end = *src as usize;

                    let mut arg_values = Vec::with_capacity(arg_end - arg_start);
                    let registers_to_copy = &register_window[arg_start..arg_end];
                    for register in registers_to_copy {
                        arg_values.push(register.clone());
                    }

                    // TODO: return value?
                    let return_value = (native_function)(arg_values);
                    if let Some(return_value) = return_value {
                        register_window[*return_val as usize] = return_value
                    }

                    ip += 1;
                }
                Instruction::CallFunction {
                    src,
                    arg_count,
                    return_val,
                } => {
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

                    let arg_start = old_base + *src as usize - *arg_count as usize;
                    let arg_end = old_base + *src as usize;
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
                        function_return_value: *return_val,
                    });

                    // tracing::warn!("register: {:?}", register_window);
                    // tracing::warn!("register: {:?}", self.global_register_count);
                    // tracing::warn!("register: {:?}", register_count);

                    Self::print_registers(register_window);
                    continue;
                }

                Instruction::LoadLiteral { dest, src } => {
                    let literal = &self.literals[*src as usize];
                    register_window[*dest as usize] =
                        RegisterValue::Literal(Cow::Borrowed(literal));

                    ip += 1;
                }

                Instruction::Add { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, +, rhs);

                    ip += 1;
                }

                Instruction::Sub { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, -, rhs);

                    ip += 1;
                }

                Instruction::Mul { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, *, rhs);

                    ip += 1;
                }

                Instruction::Div { dest, lhs, rhs } => {
                    impl_binary_op!(register_window, dest, lhs, /, rhs);

                    ip += 1;
                }

                Instruction::Equals { dest, lhs, rhs } => {
                    impl_binary_comparator!(register_window, dest, lhs, ==, rhs);

                    ip += 1;
                }

                Instruction::NotEquals { dest, lhs, rhs } => {
                    impl_binary_comparator!(register_window, dest, lhs, !=, rhs);

                    ip += 1;
                }

                Instruction::GreaterThan { dest, lhs, rhs } => {
                    impl_binary_comparator!(register_window, dest, lhs, >, rhs);

                    ip += 1;
                }

                Instruction::GreaterThanOrEquals { dest, lhs, rhs } => {
                    impl_binary_comparator!(register_window, dest, lhs, >=, rhs);

                    ip += 1;
                }

                Instruction::LessThan { dest, lhs, rhs } => {
                    impl_binary_comparator!(register_window, dest, lhs, <, rhs);

                    ip += 1;
                }

                Instruction::LessThanOrEquals { dest, lhs, rhs } => {
                    impl_binary_comparator!(register_window, dest, lhs, <=, rhs);

                    ip += 1;
                }

                Instruction::Copy { dest, src } => {
                    register_window[*dest as usize] = register_window[*src as usize].clone();

                    ip += 1;
                }
                Instruction::PrefixNot { dest, rhs } => {
                    let rhs = &register_window[*rhs as usize];

                    match rhs {
                        RegisterValue::Literal(literal) => match literal.as_ref() {
                            ast::Literal::Boolean(v) => {
                                register_window[*dest as usize] =
                                    RegisterValue::Literal(Cow::Owned(Literal::Boolean(!v)))
                            }

                            _ => {
                                return Err(ExecutionError::InvalidOperation {
                                    cause: "cannot use '!' on non boolean type".to_owned(),
                                })
                            }
                        },
                        _ => {
                            return Err(ExecutionError::InvalidOperation {
                                cause: "'!' must be used on literals only".to_owned(),
                            })
                        }
                    }

                    ip += 1;
                }
                Instruction::PrefixSub { dest, rhs } => {
                    let rhs = &register_window[*rhs as usize];

                    match rhs {
                        RegisterValue::Literal(literal) => match literal.as_ref() {
                            ast::Literal::Float(v) => {
                                let new_value = -(*v);
                                register_window[*dest as usize] =
                                    RegisterValue::Literal(Cow::Owned(Literal::Float(new_value)))
                            }

                            ast::Literal::Integer(v) => {
                                let new_value = -(*v);
                                register_window[*dest as usize] =
                                    RegisterValue::Literal(Cow::Owned(Literal::Integer(new_value)))
                            }

                            _ => {
                                return Err(ExecutionError::InvalidOperation {
                                    cause: "'-' must be used on number types".to_owned(),
                                })
                            }
                        },
                        _ => {
                            return Err(ExecutionError::InvalidOperation {
                                cause: "'-' must be used on literals only".to_owned(),
                            })
                        }
                    }

                    ip += 1;
                }
                Instruction::JumpIfFalse { src, offset } => {
                    let register_value = &register_window[*src as usize];
                    // FIXME: are we type checked?
                    match register_value {
                        RegisterValue::Function(_) | RegisterValue::Empty => unreachable!(),
                        RegisterValue::Literal(l) => match l.as_ref() {
                            Literal::Boolean(b) => {
                                if *b {
                                    ip += 1;
                                } else {
                                    ip += *offset as usize;
                                }
                            }
                            _ => unreachable!(),
                        },
                    }
                }
                Instruction::Jump { offset } => ip += *offset as usize,
                Instruction::JumpReverse { offset } => ip -= *offset as usize,
            }

            Self::print_registers(register_window);
        }

        // dbg!(registers);

        Ok(registers)
    }

    pub fn run(&self) -> Result<(), ExecutionError> {
        self.run_with_registers_returned().map(|_| ())
    }
}
