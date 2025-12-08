use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub functions: Vec<IRFunction>,
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub param_count: usize,
    pub local_count: usize,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    LoadConst(IRValue),
    LoadLocal(usize),
    StoreLocal(usize),
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
    Call(String, usize),
    Jump(usize),
    JumpIfFalse(usize),
    Return,
    Pop,
    Print,
}

#[derive(Debug, Clone)]
pub enum IRValue {
    Int(i64),
    Bool(bool),
    String(String),
}

pub struct IRGenerator {
    local_map: HashMap<String, usize>,
    next_local: usize,
    instructions: Vec<IRInstruction>,
}

impl IRGenerator {
    pub fn new() -> Self {
        Self {
            local_map: HashMap::new(),
            next_local: 0,
            instructions: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<IRProgram, String> {
        let mut functions = Vec::new();

        for function in &program.functions {
            functions.push(self.generate_function(function)?);
        }

        Ok(IRProgram { functions })
    }

    fn generate_function(&mut self, function: &FunctionDecl) -> Result<IRFunction, String> {
        self.local_map.clear();
        self.next_local = 0;
        self.instructions.clear();

        for param in &function.parameters {
            self.declare_local(param.name.clone());
        }

        let param_count = function.parameters.len();

        for statement in &function.body {
            self.generate_statement(statement)?;
        }

        if function.return_type == Type::Void {
            self.instructions.push(IRInstruction::LoadConst(IRValue::Int(0)));
            self.instructions.push(IRInstruction::Return);
        }

        Ok(IRFunction {
            name: function.name.clone(),
            param_count,
            local_count: self.next_local,
            instructions: self.instructions.clone(),
        })
    }

    fn generate_statement(&mut self, statement: &Statement) -> Result<(), String> {
    match statement {
        Statement::LetDecl { name, initializer, .. } => {
            if let Some(init) = initializer {
                self.generate_expression(init)?;
            } else {
                self.instructions.push(IRInstruction::LoadConst(IRValue::Int(0)));
            }
            let local_idx = self.declare_local(name.clone());
            self.instructions.push(IRInstruction::StoreLocal(local_idx));
        }
        Statement::Assignment { name, value } => {
            self.generate_expression(value)?;
            let local_idx = self.lookup_local(name)?;
            self.instructions.push(IRInstruction::StoreLocal(local_idx));
        }
        Statement::If { condition, then_branch, else_branch } => {
            self.generate_expression(condition)?;

            let jump_to_else = self.instructions.len();
            self.instructions.push(IRInstruction::JumpIfFalse(0));

            for stmt in then_branch {
                self.generate_statement(stmt)?;
            }

            if let Some(else_stmts) = else_branch {
                let jump_to_end = self.instructions.len();
                self.instructions.push(IRInstruction::Jump(0));

                let else_start = self.instructions.len();
                self.instructions[jump_to_else] = IRInstruction::JumpIfFalse(else_start);

                for stmt in else_stmts {
                    self.generate_statement(stmt)?;
                }

                let end_pos = self.instructions.len();
                self.instructions[jump_to_end] = IRInstruction::Jump(end_pos);
            } else {
                let end_pos = self.instructions.len();
                self.instructions[jump_to_else] = IRInstruction::JumpIfFalse(end_pos);
            }
        }
        Statement::While { condition, body } => {
            let loop_start = self.instructions.len();

            self.generate_expression(condition)?;

            let jump_to_end = self.instructions.len();
            self.instructions.push(IRInstruction::JumpIfFalse(0));

            for stmt in body {
                self.generate_statement(stmt)?;
            }

            self.instructions.push(IRInstruction::Jump(loop_start));

            let end_pos = self.instructions.len();
            self.instructions[jump_to_end] = IRInstruction::JumpIfFalse(end_pos);
        }
        Statement::Return { value } => {
            if let Some(expr) = value {
                self.generate_expression(expr)?;
            } else {
                self.instructions.push(IRInstruction::LoadConst(IRValue::Int(0)));
            }
            self.instructions.push(IRInstruction::Return);
        }
        Statement::Print { expression } => {
            self.generate_expression(expression)?;
            self.instructions.push(IRInstruction::Print);
        }
        Statement::Expression { expression } => {
            self.generate_expression(expression)?;
            self.instructions.push(IRInstruction::Pop);
        }
    }

    Ok(())
}

    fn generate_expression(&mut self, expression: &Expression) -> Result<(), String> {
        match expression {
            Expression::IntLiteral(value) => {
                self.instructions.push(IRInstruction::LoadConst(IRValue::Int(*value)));
            }
            Expression::BoolLiteral(value) => {
                self.instructions.push(IRInstruction::LoadConst(IRValue::Bool(*value)));
            }
            Expression::StringLiteral(value) => {
                self.instructions.push(IRInstruction::LoadConst(IRValue::String(value.clone())));
            }
            Expression::Variable(name) => {
                let local_idx = self.lookup_local(name)?;
                self.instructions.push(IRInstruction::LoadLocal(local_idx));
            }
            Expression::Binary { left, operator, right } => {
                self.generate_expression(left)?;
                self.generate_expression(right)?;

                let instruction = match operator {
                    BinaryOp::Add => IRInstruction::Add,
                    BinaryOp::Subtract => IRInstruction::Sub,
                    BinaryOp::Multiply => IRInstruction::Mul,
                    BinaryOp::Divide => IRInstruction::Div,
                    BinaryOp::Modulo => IRInstruction::Mod,
                    BinaryOp::Equal => IRInstruction::Eq,
                    BinaryOp::NotEqual => IRInstruction::Ne,
                    BinaryOp::Less => IRInstruction::Lt,
                    BinaryOp::LessEqual => IRInstruction::Le,
                    BinaryOp::Greater => IRInstruction::Gt,
                    BinaryOp::GreaterEqual => IRInstruction::Ge,
                    BinaryOp::And => IRInstruction::And,
                    BinaryOp::Or => IRInstruction::Or,
                };

                self.instructions.push(instruction);
            }
            Expression::Unary { operator, operand } => {
                self.generate_expression(operand)?;

                let instruction = match operator {
                    UnaryOp::Negate => IRInstruction::Neg,
                    UnaryOp::Not => IRInstruction::Not,
                };

                self.instructions.push(instruction);
            }
            Expression::Call { function, arguments } => {
                for arg in arguments {
                    self.generate_expression(arg)?;
                }
                self.instructions.push(IRInstruction::Call(function.clone(), arguments.len()));
            }
        }

        Ok(())
    }

    fn declare_local(&mut self, name: String) -> usize {
        if let Some(&idx) = self.local_map.get(&name) {
            return idx;
        }
        let idx = self.next_local;
        self.local_map.insert(name, idx);
        self.next_local += 1;
        idx
    }

    fn lookup_local(&self, name: &str) -> Result<usize, String> {
        self.local_map.get(name)
            .copied()
            .ok_or(format!("Undefined variable '{}'", name))
    }
}