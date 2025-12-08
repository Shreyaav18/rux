use crate::bytecode::*;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
}

impl Value {
    fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(v) => Ok(*v),
            _ => Err(format!("Expected int, got {:?}", self)),
        }
    }

    fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(v) => Ok(*v),
            _ => Err(format!("Expected bool, got {:?}", self)),
        }
    }
}

struct CallFrame {
    function_name: String,
    ip: usize,
    locals: Vec<Value>,
}

pub struct VM {
    program: BytecodeProgram,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    current_function: String,
    ip: usize,
    locals: Vec<Value>,
}

impl VM {
    pub fn new(program: BytecodeProgram) -> Self {
        Self {
            program,
            stack: Vec::new(),
            call_stack: Vec::new(),
            current_function: String::new(),
            ip: 0,
            locals: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        let main_func = self.program.main_function.clone();
        self.call_function(&main_func, 0)?;
        self.execute()?;
        Ok(())
    }

    fn call_function(&mut self, func_name: &str, arg_count: usize) -> Result<(), String> {
    let function = self.program.functions.get(func_name)
        .ok_or(format!("Undefined function '{}'", func_name))?
        .clone();

    if arg_count != function.param_count {
        return Err(format!(
            "Function '{}' expects {} arguments, got {}",
            func_name, function.param_count, arg_count
        ));
    }

    if !self.current_function.is_empty() {
        self.call_stack.push(CallFrame {
            function_name: self.current_function.clone(),
            ip: self.ip,
            locals: self.locals.clone(),
        });
    }

    let mut new_locals = vec![Value::Int(0); function.local_count];
    
    let mut args = Vec::new();
    for _ in 0..arg_count {
        args.push(self.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();
    
    for (i, arg) in args.into_iter().enumerate() {
        new_locals[i] = arg;
    }

    self.current_function = func_name.to_string();
    self.ip = 0;
    self.locals = new_locals;

    Ok(())
}

    fn execute(&mut self) -> Result<(), String> {
    loop {
        let function = self.program.functions.get(&self.current_function)
            .ok_or("Current function not found")?
            .clone();

        if self.ip >= function.code.len() {
            return Err("Instruction pointer out of bounds".to_string());
        }

        let opcode = function.code[self.ip];
        self.ip += 1;

        match opcode {
            op if op == OpCode::LoadConst as u8 => {
                let const_idx = self.read_u16(&function.code)?;
                let constant = &function.constants[const_idx as usize];
                let value = match constant {
                    BytecodeValue::Int(v) => Value::Int(*v),
                    BytecodeValue::Bool(v) => Value::Bool(*v),
                    BytecodeValue::String(v) => Value::String(v.clone()),
                };
                self.stack.push(value);
            }
            op if op == OpCode::LoadLocal as u8 => {
                let local_idx = self.read_u16(&function.code)? as usize;
            
                if local_idx >= self.locals.len() {
                    return Err(format!("Local variable {} out of bounds (have {} locals)", local_idx, self.locals.len()));
                }
                let value = self.locals[local_idx].clone();
                self.stack.push(value);
            }
            op if op == OpCode::StoreLocal as u8 => {
                let local_idx = self.read_u16(&function.code)? as usize;
                let value = self.stack.pop().ok_or("Stack underflow")?;
            
                if local_idx >= self.locals.len() {
                    self.locals.resize(local_idx + 1, Value::Int(0));
                }
                self.locals[local_idx] = value;
            }
            op if op == OpCode::Add as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow in Add (right)")?;
                let a = self.stack.pop().ok_or("Stack underflow in Add (left)")?;
                self.stack.push(Value::Int(a.as_int()? + b.as_int()?));
            }
            op if op == OpCode::Sub as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow in Sub")?;
                let a = self.stack.pop().ok_or("Stack underflow in Sub")?;
                self.stack.push(Value::Int(a.as_int()? - b.as_int()?));
            }
            op if op == OpCode::Mul as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow in Mul")?;
                let a = self.stack.pop().ok_or("Stack underflow in Mul")?;
                self.stack.push(Value::Int(a.as_int()? * b.as_int()?));
            }
            op if op == OpCode::Div as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(Value::Int(a / b));
            }
            op if op == OpCode::Mod as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                if b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                self.stack.push(Value::Int(a % b));
            }
            op if op == OpCode::Eq as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                let result = match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x == y,
                    (Value::Bool(x), Value::Bool(y)) => x == y,
                    (Value::String(x), Value::String(y)) => x == y,
                    _ => false,
                };
                self.stack.push(Value::Bool(result));
            }
            op if op == OpCode::Ne as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                let result = match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x != y,
                    (Value::Bool(x), Value::Bool(y)) => x != y,
                    (Value::String(x), Value::String(y)) => x != y,
                    _ => true,
                };
                self.stack.push(Value::Bool(result));
            }
            op if op == OpCode::Lt as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                self.stack.push(Value::Bool(a < b));
            }
            op if op == OpCode::Le as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                self.stack.push(Value::Bool(a <= b));
            }
            op if op == OpCode::Gt as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                self.stack.push(Value::Bool(a > b));
            }
            op if op == OpCode::Ge as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                self.stack.push(Value::Bool(a >= b));
            }
            op if op == OpCode::And as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_bool()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_bool()?;
                self.stack.push(Value::Bool(a && b));
            }
            op if op == OpCode::Or as u8 => {
                let b = self.stack.pop().ok_or("Stack underflow")?.as_bool()?;
                let a = self.stack.pop().ok_or("Stack underflow")?.as_bool()?;
                self.stack.push(Value::Bool(a || b));
            }
            op if op == OpCode::Neg as u8 => {
                let a = self.stack.pop().ok_or("Stack underflow")?.as_int()?;
                self.stack.push(Value::Int(-a));
            }
            op if op == OpCode::Not as u8 => {
                let a = self.stack.pop().ok_or("Stack underflow")?.as_bool()?;
                self.stack.push(Value::Bool(!a));
            }
            op if op == OpCode::Call as u8 => {
                let name_len = function.code[self.ip] as usize;
                self.ip += 1;
                let name_bytes = &function.code[self.ip..self.ip + name_len];
                self.ip += name_len;
                let func_name = String::from_utf8(name_bytes.to_vec())
                    .map_err(|_| "Invalid function name")?;
                let arg_count = function.code[self.ip] as usize;
                self.ip += 1;

                self.call_function(&func_name, arg_count)?;
                continue;
            }
            op if op == OpCode::Jump as u8 => {
                let target = self.read_u32(&function.code)? as usize;
                self.ip = target;
            }
            op if op == OpCode::JumpIfFalse as u8 => {
                let target = self.read_u32(&function.code)? as usize;
                let condition = self.stack.pop().ok_or("Stack underflow in JumpIfFalse")?.as_bool()?;
                if !condition {
                    self.ip = target;
                }
            }
            op if op == OpCode::Return as u8 => {
                let return_value = self.stack.pop().ok_or("Stack underflow in Return")?;

                if let Some(frame) = self.call_stack.pop() {
                    self.current_function = frame.function_name;
                    self.ip = frame.ip;
                    self.locals = frame.locals;
                    self.stack.push(return_value);
                } else {
                    return Ok(());
                }
            }
            op if op == OpCode::Pop as u8 => {
                self.stack.pop().ok_or("Stack underflow in Pop")?;
            }
            op if op == OpCode::Print as u8 => {
                let value = self.stack.pop().ok_or("Stack underflow in Print")?;
                match value {
                    Value::Int(v) => println!("{}", v),
                    Value::Bool(v) => println!("{}", v),
                    Value::String(v) => println!("{}", v),
                }
            }
            _ => return Err(format!("Unknown opcode: {}", opcode)),
        }
    }
}

    fn read_u16(&mut self, code: &[u8]) -> Result<u16, String> {
        if self.ip + 1 >= code.len() {
            return Err("Unexpected end of bytecode".to_string());
        }
        let bytes = [code[self.ip], code[self.ip + 1]];
        self.ip += 2;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self, code: &[u8]) -> Result<u32, String> {
        if self.ip + 3 >= code.len() {
            return Err("Unexpected end of bytecode".to_string());
        }
        let bytes = [code[self.ip], code[self.ip + 1], code[self.ip + 2], code[self.ip + 3]];
        self.ip += 4;
        Ok(u32::from_le_bytes(bytes))
    }
}