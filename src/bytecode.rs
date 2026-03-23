use crate::ir::*;
use crate::token::{CompileError, CompilePhase};
use std::collections::HashMap;

// ── Public output types ───────────────────────────────────────────────────────

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

// ── Constant-pool key ─────────────────────────────────────────────────────────
//
// Using a typed enum instead of `format!("{:?}", value)` avoids a heap
// allocation per instruction and prevents collisions between e.g.
// Int(1) and a hypothetical string "Int(1)".

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConstKey {
    Int(i64),
    Bool(bool),
    /// String constants are interned by value.
    String(String),
}

impl From<&IRValue> for ConstKey {
    fn from(v: &IRValue) -> Self {
        match v {
            IRValue::Int(n)    => ConstKey::Int(*n),
            IRValue::Bool(b)   => ConstKey::Bool(*b),
            IRValue::String(s) => ConstKey::String(s.clone()),
        }
    }
}

// ── Generator ─────────────────────────────────────────────────────────────────

pub struct BytecodeGenerator;

impl BytecodeGenerator {
    pub fn generate(ir_program: &IRProgram) -> Result<BytecodeProgram, CompileError> {
        let mut functions = HashMap::new();
        let mut main_function: Option<String> = None;

        for ir_func in &ir_program.functions {
            let bytecode_func = Self::generate_function(ir_func)?;

            if ir_func.name == "main" {
                main_function = Some(ir_func.name.clone());
            }

            functions.insert(ir_func.name.clone(), bytecode_func);
        }

        let main_function = main_function.ok_or_else(|| CompileError::new(
            CompilePhase::Codegen,
            "no 'main' function found",
            0, 0,
        ))?;

        Ok(BytecodeProgram { functions, main_function })
    }

    fn generate_function(ir_func: &IRFunction) -> Result<BytecodeFunction, CompileError> {
        let mut code: Vec<u8>                    = Vec::new();
        let mut constants: Vec<BytecodeValue>    = Vec::new();
        let mut constant_map: HashMap<ConstKey, usize> = HashMap::new();
        // (byte-offset of the placeholder, target instruction index)
        let mut jump_patches: Vec<(usize, usize)>= Vec::new();
        // instruction index → byte offset in `code`
        let mut instruction_offsets: Vec<usize>  = Vec::new();

        for instruction in &ir_func.instructions {
            instruction_offsets.push(code.len());

            match instruction {
                IRInstruction::LoadConst(value) => {
                    let key = ConstKey::from(value);
                    let const_idx = *constant_map.entry(key).or_insert_with(|| {
                        let idx = constants.len();
                        constants.push(match value {
                            IRValue::Int(v)    => BytecodeValue::Int(*v),
                            IRValue::Bool(v)   => BytecodeValue::Bool(*v),
                            IRValue::String(v) => BytecodeValue::String(v.clone()),
                        });
                        idx
                    });

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

                IRInstruction::Add  => code.push(OpCode::Add  as u8),
                IRInstruction::Sub  => code.push(OpCode::Sub  as u8),
                IRInstruction::Mul  => code.push(OpCode::Mul  as u8),
                IRInstruction::Div  => code.push(OpCode::Div  as u8),
                IRInstruction::Mod  => code.push(OpCode::Mod  as u8),
                IRInstruction::Eq   => code.push(OpCode::Eq   as u8),
                IRInstruction::Ne   => code.push(OpCode::Ne   as u8),
                IRInstruction::Lt   => code.push(OpCode::Lt   as u8),
                IRInstruction::Le   => code.push(OpCode::Le   as u8),
                IRInstruction::Gt   => code.push(OpCode::Gt   as u8),
                IRInstruction::Ge   => code.push(OpCode::Ge   as u8),
                IRInstruction::And  => code.push(OpCode::And  as u8),
                IRInstruction::Or   => code.push(OpCode::Or   as u8),
                IRInstruction::Neg  => code.push(OpCode::Neg  as u8),
                IRInstruction::Not  => code.push(OpCode::Not  as u8),
                IRInstruction::Print => code.push(OpCode::Print as u8),
                IRInstruction::Return => code.push(OpCode::Return as u8),
                IRInstruction::Pop  => code.push(OpCode::Pop  as u8),

                IRInstruction::Call(func_name, arg_count) => {
                    let name_bytes = func_name.as_bytes();
                    if name_bytes.len() > u8::MAX as usize {
                        return Err(CompileError::new(
                            CompilePhase::Codegen,
                            format!("function name '{}' exceeds 255 bytes", func_name),
                            0, 0,
                        ));
                    }
                    code.push(OpCode::Call as u8);
                    code.push(name_bytes.len() as u8);
                    code.extend_from_slice(name_bytes);
                    code.push(*arg_count as u8);
                }

                IRInstruction::Jump(target) => {
                    code.push(OpCode::Jump as u8);
                    let patch_loc = code.len();
                    code.extend_from_slice(&0u32.to_le_bytes());
                    jump_patches.push((patch_loc, *target));
                }

                IRInstruction::JumpIfFalse(target) => {
                    code.push(OpCode::JumpIfFalse as u8);
                    let patch_loc = code.len();
                    code.extend_from_slice(&0u32.to_le_bytes());
                    jump_patches.push((patch_loc, *target));
                }
            }
        }

        // Back-patch jump targets now that all byte offsets are known.
        for (patch_loc, target_instr) in jump_patches {
            let target_offset = instruction_offsets
                .get(target_instr)
                .copied()
                .unwrap_or(code.len()); // jump past end = fall off function

            code[patch_loc..patch_loc + 4].copy_from_slice(&(target_offset as u32).to_le_bytes());
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // Stack / locals
    LoadConst,
    LoadLocal,
    StoreLocal,
    Pop,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
    Neg,
    Not,

    // Control flow
    Jump,
    JumpIfFalse,
    Return,

    // Misc
    Call,
    Print,
}