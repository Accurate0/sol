use crate::compiler;
use crate::types::{Array, Literal, Object, ObjectValue};
use crate::{
    compiler::CompiledProgram,
    impl_binary_comparator, impl_binary_op,
    instructions::Instruction,
    stdlib::{NativeFunctionType, STANDARD_LIBRARY},
    types,
};
use std::rc::Rc;
use std::{borrow::Cow, collections::HashMap};
use thiserror::Error;

mod registers;
mod value;
pub use registers::*;
pub use value::*;

struct SavedCallFrame {
    // FIXME: we could store the register array with each callframe
    // right now each function is allowed 256 registers by the compiler
    // but the VM only has 256 total, a deep call stack will easily go
    // beyond this number, we'd need more memory but could allocate
    // the right number of registers for each function
    // or make the window resizable... abstract the window away and resize on access
    pub ip: usize,
    pub function: VMFunction,
    pub register_count: u8,
    pub function_return_value: u8,
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("{cause}")]
    InvalidOperation { cause: String },
}

pub struct VM {
    functions: Vec<VMFunction>,
    native_functions: HashMap<String, NativeFunctionType>,
    global_function: VMFunction,
    literals: Vec<types::Literal>,
}

impl VM {
    pub fn new(compiled_program: CompiledProgram) -> Self {
        Self {
            functions: compiled_program
                .functions
                .into_iter()
                .map(Rc::new)
                .collect(),
            native_functions: Default::default(),
            global_function: compiler::Function {
                name: "global".to_owned(),
                code: compiled_program.global_code,
                register_count: compiled_program.global_register_count,
            }
            .into(),
            literals: compiled_program.literals,
        }
    }

    #[allow(unused)]
    pub fn define_native_function(mut self, name: String, function: NativeFunctionType) -> Self {
        self.native_functions.insert(name, function);

        self
    }

    fn print_registers(window: &Registers<'_>) {
        for (i, item) in window.regs().iter().enumerate() {
            match item {
                VMValue::Empty => {}
                VMValue::Literal(l) => tracing::debug!("{i} {:?}", l),
                VMValue::Function(f) => tracing::debug!("{i} {:?}", f.name),
                VMValue::Object(object) => tracing::debug!("{i} {:?}", object),
                VMValue::Array(array) => tracing::debug!("{i} {:?}", array),
            }
        }

        tracing::debug!("");
    }

    pub fn run_with_registers_returned(&self) -> Result<Registers, ExecutionError> {
        let mut registers = Registers::default();

        let mut saved_call_frames = Vec::<SavedCallFrame>::new();
        let mut current_function = self.global_function.clone();

        let mut ip = 0;
        loop {
            if ip >= current_function.code.len() {
                break;
            }

            let current_instruction = current_function.code[ip];
            tracing::debug!("executing: {:?}", current_instruction);
            // tracing::info!("ip: {:?}", ip);
            // tracing::info!("code: {:?}", current_code);
            // tracing::info!("reg: {:?}", registers);
            // tracing::info!("base_reg: {:?}", base_register);

            match current_instruction {
                Instruction::FunctionReturn => {
                    if let Some(saved_call_frame) = saved_call_frames.pop() {
                        let mut base_register = registers.base_register();
                        if let Some(current_call_frame) = saved_call_frames.last() {
                            base_register -= current_call_frame.register_count as usize;
                        } else {
                            base_register -= self.global_function.register_count as usize;
                        }

                        registers.update_base_register(base_register);

                        ip = saved_call_frame.ip + 1;
                        current_function = saved_call_frame.function;
                        continue;
                    };
                }
                Instruction::Return { val } => {
                    if let Some(saved_call_frame) = saved_call_frames.pop() {
                        let mut base_register = registers.base_register();
                        if let Some(current_call_frame) = saved_call_frames.last() {
                            base_register -= current_call_frame.register_count as usize;
                        } else {
                            base_register -= self.global_function.register_count as usize;
                        }

                        let register_to_copy_to = saved_call_frame.function_return_value;
                        let register_to_copy_from = val;

                        let from = registers[register_to_copy_from].clone();

                        registers.update_base_register(base_register);

                        registers[register_to_copy_to] = from;

                        ip = saved_call_frame.ip + 1;
                        current_function = saved_call_frame.function;
                        continue;
                    };
                }
                Instruction::LoadFunction { dest, src } => {
                    let func = self.functions[src as usize].clone();
                    registers[dest] = VMValue::Function(func);

                    ip += 1;
                }
                Instruction::CallNativeFunction {
                    src,
                    arg_count,
                    return_val,
                } => {
                    let register = &registers[src];
                    let function_name = match register {
                        VMValue::Literal(cow) => match cow.as_ref() {
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

                    let arg_start = src - arg_count;
                    let arg_end = src;

                    let mut arg_values = Vec::with_capacity((arg_end - arg_start) as usize);
                    let registers_to_copy = &registers[arg_start..arg_end];
                    for register in registers_to_copy {
                        arg_values.push(register.clone());
                    }

                    // TODO: return value?
                    let return_value = (native_function)(arg_values);
                    if let Some(return_value) = return_value {
                        registers[return_val] = return_value
                    }

                    ip += 1;
                }
                Instruction::CallFunction {
                    src,
                    arg_count,
                    return_val,
                } => {
                    let func = &registers[src];
                    let func = match func {
                        VMValue::Function(f) => f.clone(),
                        _ => unreachable!(),
                    };

                    // eprintln!("DEBUGPRINT[2]: vm.rs:123: arg_start={:#?}", arg_start);
                    // eprintln!("DEBUGPRINT[3]: vm.rs:124: arg_end={:#?}", arg_end);
                    // tracing::info!("func: {:?}", func);

                    let old_function = current_function;
                    let old_ip = ip;

                    current_function = func.clone();
                    ip = 0;

                    let mut base_register = registers.base_register();
                    let old_base = base_register;
                    let register_count = func.register_count;
                    if let Some(current_call_frame) = saved_call_frames.last() {
                        base_register += current_call_frame.register_count as usize;
                    } else {
                        base_register += self.global_function.register_count as usize;
                    }

                    registers.update_base_register(base_register);

                    let (old_function_regs, new_function_regs) =
                        registers.regs_mut().split_at_mut(base_register);

                    // tracing::warn!("FUNCTION CALL: OLD");
                    // Self::print_registers(old_function);

                    let arg_start = old_base + src as usize - arg_count as usize;
                    let arg_end = old_base + src as usize;
                    let registers_to_copy = &old_function_regs[arg_start..arg_end];

                    // tracing::warn!("FUNCTION CALL: COPY");
                    // Self::print_registers(registers_to_copy);

                    for (index, register) in registers_to_copy.iter().enumerate() {
                        new_function_regs[index + 1] = register.clone();
                    }

                    // tracing::warn!("FUNCTION CALL: NEW");
                    // Self::print_registers(new_function);

                    registers.update_base_register(base_register);
                    saved_call_frames.push(SavedCallFrame {
                        ip: old_ip,
                        function: old_function,
                        register_count,
                        function_return_value: return_val,
                    });

                    // tracing::warn!("register: {:?}", registers);
                    // tracing::warn!("register: {:?}", self.global_register_count);
                    // tracing::warn!("register: {:?}", register_count);

                    Self::print_registers(&registers);
                    continue;
                }

                Instruction::LoadLiteral { dest, src } => {
                    let literal = &self.literals[src as usize];
                    registers[dest] = VMValue::Literal(Cow::Borrowed(literal));

                    ip += 1;
                }

                Instruction::Add { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, +, rhs);

                    ip += 1;
                }

                Instruction::Sub { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, -, rhs);

                    ip += 1;
                }

                Instruction::Mul { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, *, rhs);

                    ip += 1;
                }

                Instruction::Div { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, /, rhs);

                    ip += 1;
                }

                Instruction::Equals { dest, lhs, rhs } => {
                    impl_binary_comparator!(registers, dest, lhs, ==, rhs);

                    ip += 1;
                }

                Instruction::NotEquals { dest, lhs, rhs } => {
                    impl_binary_comparator!(registers, dest, lhs, !=, rhs);

                    ip += 1;
                }

                Instruction::GreaterThan { dest, lhs, rhs } => {
                    impl_binary_comparator!(registers, dest, lhs, >, rhs);

                    ip += 1;
                }

                Instruction::GreaterThanOrEquals { dest, lhs, rhs } => {
                    impl_binary_comparator!(registers, dest, lhs, >=, rhs);

                    ip += 1;
                }

                Instruction::LessThan { dest, lhs, rhs } => {
                    impl_binary_comparator!(registers, dest, lhs, <, rhs);

                    ip += 1;
                }

                Instruction::LessThanOrEquals { dest, lhs, rhs } => {
                    impl_binary_comparator!(registers, dest, lhs, <=, rhs);

                    ip += 1;
                }

                Instruction::Copy { dest, src } => {
                    registers[dest] = registers[src].clone();

                    ip += 1;
                }
                Instruction::PrefixNot { dest, rhs } => {
                    let rhs = &registers[rhs];

                    match rhs {
                        VMValue::Literal(literal) => match literal.as_ref() {
                            types::Literal::Boolean(v) => {
                                registers[dest] = VMValue::Literal(Cow::Owned(Literal::Boolean(!v)))
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
                    let rhs = &registers[rhs];

                    match rhs {
                        VMValue::Literal(literal) => match literal.as_ref() {
                            types::Literal::Float(v) => {
                                let new_value = -(*v);
                                registers[dest] =
                                    VMValue::Literal(Cow::Owned(Literal::Float(new_value)))
                            }

                            types::Literal::Integer(v) => {
                                let new_value = -(*v);
                                registers[dest] =
                                    VMValue::Literal(Cow::Owned(Literal::Integer(new_value)))
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
                    let register_value = &registers[src];
                    // FIXME: are we type checked?
                    match register_value {
                        VMValue::Object(_)
                        | VMValue::Function(_)
                        | VMValue::Empty
                        | VMValue::Array(_) => {
                            unreachable!()
                        }
                        VMValue::Literal(l) => match l.as_ref() {
                            Literal::Boolean(b) => {
                                if *b {
                                    ip += 1;
                                } else {
                                    ip += offset as usize;
                                }
                            }
                            _ => unreachable!(),
                        },
                    }
                }
                Instruction::Jump { offset } => ip += offset as usize,
                Instruction::JumpReverse { offset } => ip -= offset as usize,
                Instruction::AllocateObject { dest } => {
                    registers[dest] = VMValue::Object(Object::create_for_vm());
                    ip += 1;
                }
                Instruction::SetObjectField {
                    object,
                    field,
                    value,
                } => {
                    let obj = match &registers[object] {
                        VMValue::Object(object) => object,
                        _ => unreachable!(),
                    };

                    let key = match &registers[field] {
                        VMValue::Literal(lit) => match lit.as_ref() {
                            Literal::String(s) => s.clone(),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };

                    let value = match &registers[value] {
                        VMValue::Literal(lit) => ObjectValue::Literal(lit.as_ref().clone()),
                        VMValue::Object(object) => ObjectValue::Object(object.clone()),
                        VMValue::Function(f) => ObjectValue::Function(f.clone()),
                        VMValue::Array(array) => ObjectValue::Array(array.clone()),
                        _ => unreachable!(),
                    };

                    obj.borrow_mut().insert(key, Rc::new(value.into()));
                    ip += 1;
                }
                Instruction::GetObjectField {
                    object,
                    field,
                    return_val,
                } => {
                    let key = match &registers[field] {
                        VMValue::Literal(lit) => lit.as_ref(),
                        _ => unreachable!(),
                    };

                    let register_value = {
                        let obj = match registers[object] {
                            VMValue::Object(ref object) => object.clone(),
                            _ => unreachable!(),
                        };
                        let obj = obj.borrow();
                        let obj_value = obj.index(key);

                        match obj_value {
                            Some(obj) => {
                                let obj = obj.clone();
                                let obj = obj.borrow();

                                match &*obj {
                                    ObjectValue::Object(rc) => VMValue::Object(rc.clone()),
                                    ObjectValue::Literal(literal) => {
                                        VMValue::Literal(Cow::Owned(literal.clone()))
                                    }
                                    ObjectValue::Function(func) => VMValue::Function(func.clone()),
                                    ObjectValue::Array(rc) => VMValue::Array(rc.clone()),
                                    ObjectValue::Nil => VMValue::Empty,
                                }
                            }
                            None => VMValue::Empty,
                        }
                    };

                    registers[return_val] = register_value;
                    ip += 1;
                }
                Instruction::AllocateArray { dest } => {
                    registers[dest] = VMValue::Array(Array::create_for_vm());
                    ip += 1;
                }
                Instruction::SetArrayIndex {
                    array,
                    index,
                    value,
                } => {
                    let index = match &registers[index] {
                        VMValue::Literal(lit) => match lit.as_ref() {
                            Literal::Integer(integer) => integer,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };

                    let array = match registers[array] {
                        VMValue::Array(ref object) => object.clone(),
                        _ => unreachable!(),
                    };

                    let value = match &registers[value] {
                        VMValue::Literal(lit) => ObjectValue::Literal(lit.as_ref().clone()),
                        VMValue::Object(object) => ObjectValue::Object(object.clone()),
                        VMValue::Function(f) => ObjectValue::Function(f.clone()),
                        VMValue::Array(array) => ObjectValue::Array(array.clone()),
                        _ => unreachable!(),
                    };

                    array
                        .borrow_mut()
                        .set((*index) as usize, Rc::new(value.into()));

                    ip += 1;
                }
                Instruction::GetArrayIndex {
                    array,
                    index,
                    return_val,
                } => {
                    let index = match &registers[index] {
                        VMValue::Literal(lit) => match lit.as_ref() {
                            Literal::Integer(integer) => integer,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };

                    let register_value = {
                        let array = match registers[array] {
                            VMValue::Array(ref a) => a.clone(),
                            _ => unreachable!(),
                        };
                        let array = array.borrow();
                        let array_value = array.index((*index) as usize);

                        match array_value {
                            Some(obj) => {
                                let obj = obj.clone();
                                let obj = obj.borrow();

                                match &*obj {
                                    ObjectValue::Object(rc) => VMValue::Object(rc.clone()),
                                    ObjectValue::Literal(literal) => {
                                        VMValue::Literal(Cow::Owned(literal.clone()))
                                    }
                                    ObjectValue::Function(func) => VMValue::Function(func.clone()),
                                    ObjectValue::Array(rc) => VMValue::Array(rc.clone()),
                                    ObjectValue::Nil => VMValue::Empty,
                                }
                            }
                            None => VMValue::Empty,
                        }
                    };

                    registers[return_val] = register_value;
                    ip += 1;
                }
            }

            Self::print_registers(&registers);
        }

        // dbg!(registers);

        Ok(registers)
    }

    pub fn run(&self) -> Result<(), ExecutionError> {
        self.run_with_registers_returned().map(|_| ())
    }
}
