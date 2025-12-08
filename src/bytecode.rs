use crate::ir::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub functions: HashMap<String, BytecodeFunction>,
    pub main_function: String,
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub param_count: usize,
    pub local_count: usize,
    pub code: Vec<u8>,
    pub constants: Vec<BytecodeValue>,
}

#[derive(Debug, Clone)]
pub enum BytecodeValue {
    Int(i64),
    Bool(bool),
    String(String),
}

pub struct BytecodeGenerator;

impl BytecodeGenerator {
    pub fn generate(ir_program: &IRProgram) -> Result<BytecodeProgram, String> {
        let mut functions = HashMap::new();
        let mut main_function = String::new();

        for ir_func in &ir_program.functions {
            let bytecode_func = Self::generate_function(ir_func)?;
            
            if ir_func.name == "main" {
                main_function = ir_func.name.clone();
            }
            
            functions.insert(ir_func.name.clone(), bytecode_func);
        }

        if main_function.is_empty() {
            return Err("No main function found".to_string());
        }

        Ok(BytecodeProgram {
            functions,
            main_function,
        })
    }

    fn generate_function(ir_func: &IRFunction) -> Result<BytecodeFunction, String> {
        let mut code = Vec::new();
        let mut constants = Vec::new();
        let mut constant_map: HashMap<String, usize> = HashMap::new();
        let mut jump_patches: Vec<(usize, usize)> = Vec::new();
        let mut instruction_offsets: Vec<usize> = Vec::new();

        for (idx, instruction) in ir_func.instructions.iter().enumerate() {
            instruction_offsets.push(code.len());
            
            match instruction {
                IRInstruction::LoadConst(value) => {
                    let key = format!("{:?}", value);
                    let const_idx = if let Some(&idx) = constant_map.get(&key) {
                        idx
                    } else {
                        let idx = constants.len();
                        constants.push(match value {
                            IRValue::Int(v) => BytecodeValue::Int(*v),
                            IRValue::Bool(v) => BytecodeValue::Bool(*v),
                            IRValue::String(v) => BytecodeValue::String(v.clone()),
                        });
                        constant_map.insert(key, idx);
                        idx
                    };

                    code.push(OpCode::LoadConst as u8);
                    code.extend_from_slice(&(const_idx as u16).to_le_bytes());
                }
                IRInstruction::LoadLocal(var_idx) => {
                    code.push(OpCode::LoadLocal as u8);
                    code.extend_from_slice(&(*var_idx as u16).to_le_bytes());
                }
                IRInstruction::StoreLocal(var_idx) => {
                    code.push(OpCode::StoreLocal as u8);
                    code.extend_from_slice(&(*var_idx as u16).to_le_bytes());
                }
                IRInstruction::Add => code.push(OpCode::Add as u8),
                IRInstruction::Sub => code.push(OpCode::Sub as u8),
                IRInstruction::Mul => code.push(OpCode::Mul as u8),
                IRInstruction::Div => code.push(OpCode::Div as u8),
                IRInstruction::Mod => code.push(OpCode::Mod as u8),
                IRInstruction::Eq => code.push(OpCode::Eq as u8),
                IRInstruction::Ne => code.push(OpCode::Ne as u8),
                IRInstruction::Lt => code.push(OpCode::Lt as u8),
                IRInstruction::Le => code.push(OpCode::Le as u8),
                IRInstruction::Gt => code.push(OpCode::Gt as u8),
                IRInstruction::Ge => code.push(OpCode::Ge as u8),
                IRInstruction::And => code.push(OpCode::And as u8),
                IRInstruction::Or => code.push(OpCode::Or as u8),
                IRInstruction::Neg => code.push(OpCode::Neg as u8),
                IRInstruction::Not => code.push(OpCode::Not as u8),
                IRInstruction::Print => code.push(OpCode::Print as u8),
                IRInstruction::Call(func_name, arg_count) => {
                    let name_bytes = func_name.as_bytes();
                    code.push(OpCode::Call as u8);
                    code.push(name_bytes.len() as u8);
                    code.extend_from_slice(name_bytes);
                    code.push(*arg_count as u8);
                }
                IRInstruction::Jump(target) => {
                    code.push(OpCode::Jump as u8);
                    let patch_location = code.len();
                    code.extend_from_slice(&[0, 0, 0, 0]);
                    jump_patches.push((patch_location, *target));
                }
                IRInstruction::JumpIfFalse(target) => {
                    code.push(OpCode::JumpIfFalse as u8);
                    let patch_location = code.len();
                    code.extend_from_slice(&[0, 0, 0, 0]);
                    jump_patches.push((patch_location, *target));
                }
                IRInstruction::Return => code.push(OpCode::Return as u8),
                IRInstruction::Pop => code.push(OpCode::Pop as u8),
            }
        }

        for (patch_location, target_instruction) in jump_patches {
            let target_offset = if target_instruction < instruction_offsets.len() {
                instruction_offsets[target_instruction]
            } else {
                code.len()
            };
            
            let offset_bytes = (target_offset as u32).to_le_bytes();
            code[patch_location..patch_location + 4].copy_from_slice(&offset_bytes);
        }

        Ok(BytecodeFunction {
            name: ir_func.name.clone(),
            param_count: ir_func.param_count,
            local_count: ir_func.local_count,
            code,
            constants,
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    LoadConst,
    LoadLocal,
    StoreLocal,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Neg,
    Not,
    Call,
    Jump,
    JumpIfFalse,
    Return,
    Pop,
    Print,
}