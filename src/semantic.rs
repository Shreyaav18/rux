use crate::ast::*;
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    symbol_table: Vec<HashMap<String, Type>>,
    functions: HashMap<String, (Vec<Type>, Type)>,
    current_function_return_type: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: vec![HashMap::new()],
            functions: HashMap::new(),
            current_function_return_type: None,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        for function in &program.functions {
            let param_types: Vec<Type> = function.parameters.iter()
                .map(|p| p.param_type.clone())
                .collect();
            
            if self.functions.contains_key(&function.name) {
                return Err(format!("Function '{}' already declared", function.name));
            }
            
            self.functions.insert(
                function.name.clone(),
                (param_types, function.return_type.clone())
            );
        }

        for function in &program.functions {
            self.check_function(function)?;
        }

        if !self.functions.contains_key("main") {
            return Err("Program must have a 'main' function".to_string());
        }

        Ok(())
    }

    fn check_function(&mut self, function: &FunctionDecl) -> Result<(), String> {
        self.enter_scope();
        self.current_function_return_type = Some(function.return_type.clone());

        for param in &function.parameters {
            self.declare_variable(param.name.clone(), param.param_type.clone())?;
        }

        for statement in &function.body {
            self.check_statement(statement)?;
        }

        self.exit_scope();
        self.current_function_return_type = None;

        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::LetDecl { name, var_type, initializer } => {
                if let Some(init_expr) = initializer {
                    let expr_type = self.check_expression(init_expr)?;
                    if &expr_type != var_type {
                        return Err(format!(
                            "Type mismatch: variable '{}' declared as {:?} but initialized with {:?}",
                            name, var_type, expr_type
                        ));
                    }
                }
                self.declare_variable(name.clone(), var_type.clone())?;
            }
            Statement::Assignment { name, value } => {
                let var_type = self.lookup_variable(name)?;
                let value_type = self.check_expression(value)?;
                if var_type != value_type {
                    return Err(format!(
                        "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                        value_type, name, var_type
                    ));
                }
            }
            Statement::If { condition, then_branch, else_branch } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("If condition must be boolean, got {:?}", cond_type));
                }
                
                self.enter_scope();
                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();

                if let Some(else_stmts) = else_branch {
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                    self.exit_scope();
                }
            }
            Statement::While { condition, body } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(format!("While condition must be boolean, got {:?}", cond_type));
                }

                self.enter_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            Statement::Return { value } => {
                let return_type = self.current_function_return_type.clone()
                    .ok_or("Return statement outside of function")?;

                match (value, &return_type) {
                    (Some(_), Type::Void) => {
                        return Err("Cannot return a value from void function".to_string());
                    }
                    (None, Type::Void) => {}
                    (None, _) => {
                        return Err(format!("Function must return a value of type {:?}", return_type));
                    }
                    (Some(expr), expected_type) => {
                        let expr_type = self.check_expression(expr)?;
                        if &expr_type != expected_type {
                            return Err(format!(
                                "Return type mismatch: expected {:?}, got {:?}",
                                expected_type, expr_type
                            ));
                        }
                    }
                }
            }
            Statement::Print { expression } => {
                self.check_expression(expression)?;
            }
            Statement::Expression { expression } => {
                self.check_expression(expression)?;
            }
        }

        Ok(())
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<Type, String> {
        match expression {
            Expression::IntLiteral(_) => Ok(Type::Int),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::Variable(name) => self.lookup_variable(name),
            Expression::Binary { left, operator, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                match operator {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                        if left_type != Type::Int || right_type != Type::Int {
                            return Err(format!(
                                "Arithmetic operations require int operands, got {:?} and {:?}",
                                left_type, right_type
                            ));
                        }
                        Ok(Type::Int)
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if left_type != right_type {
                            return Err(format!(
                                "Equality comparison requires same types, got {:?} and {:?}",
                                left_type, right_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                        if left_type != Type::Int || right_type != Type::Int {
                            return Err(format!(
                                "Comparison operations require int operands, got {:?} and {:?}",
                                left_type, right_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(format!(
                                "Logical operations require bool operands, got {:?} and {:?}",
                                left_type, right_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Unary { operator, operand } => {
                let operand_type = self.check_expression(operand)?;

                match operator {
                    UnaryOp::Negate => {
                        if operand_type != Type::Int {
                            return Err(format!("Negation requires int operand, got {:?}", operand_type));
                        }
                        Ok(Type::Int)
                    }
                    UnaryOp::Not => {
                        if operand_type != Type::Bool {
                            return Err(format!("Logical not requires bool operand, got {:?}", operand_type));
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Call { function, arguments } => {
                let (param_types, return_type) = self.functions.get(function)
                    .ok_or(format!("Undefined function '{}'", function))?
                    .clone();

                if arguments.len() != param_types.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments, got {}",
                        function, param_types.len(), arguments.len()
                    ));
                }

                for (i, (arg, expected_type)) in arguments.iter().zip(param_types.iter()).enumerate() {
                    let arg_type = self.check_expression(arg)?;
                    if &arg_type != expected_type {
                        return Err(format!(
                            "Argument {} to function '{}': expected {:?}, got {:?}",
                            i + 1, function, expected_type, arg_type
                        ));
                    }
                }

                Ok(return_type)
            }
        }
    }

    fn declare_variable(&mut self, name: String, var_type: Type) -> Result<(), String> {
        let scope = self.symbol_table.last_mut().unwrap();
        if scope.contains_key(&name) {
            return Err(format!("Variable '{}' already declared in this scope", name));
        }
        scope.insert(name, var_type);
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Result<Type, String> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(var_type) = scope.get(name) {
                return Ok(var_type.clone());
            }
        }
        Err(format!("Undefined variable '{}'", name))
    }

    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.symbol_table.pop();
    }
}