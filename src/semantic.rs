use crate::ast::*;
use crate::token::{CompileError, CompilePhase};
use std::collections::HashMap;

/// One entry in the function registry.
#[derive(Clone)]
struct FunctionSig {
    param_types: Vec<Type>,
    return_type: Type,
}

// ── Analyzer ──────────────────────────────────────────────────────────────────

pub struct SemanticAnalyzer {
    /// Lexical scopes, innermost last.
    symbol_table: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionSig>,
    /// Return type of the function currently being checked.
    current_return_type: Option<Type>,
    /// Whether we are currently inside a loop (for break/continue validation).
    loop_depth: usize,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self {
            symbol_table: vec![HashMap::new()],
            functions: HashMap::new(),
            current_return_type: None,
            loop_depth: 0,
        }
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Public entry point ────────────────────────────────────────────────────

    pub fn analyze(&mut self, program: &Program) -> Result<(), CompileError> {
        // First pass: register all function signatures so forward calls work.
        for function in &program.functions {
            if self.functions.contains_key(&function.name) {
                return Err(self.error(format!("function '{}' is already declared", function.name)));
            }
            self.functions.insert(function.name.clone(), FunctionSig {
                param_types: function.parameters.iter().map(|p| p.param_type.clone()).collect(),
                return_type: function.return_type.clone(),
            });
        }

        // Second pass: type-check bodies.
        for function in &program.functions {
            self.check_function(function)?;
        }

        if !self.functions.contains_key("main") {
            return Err(self.error("program must have a 'main' function"));
        }

        Ok(())
    }

    // ── Function & statement checking ─────────────────────────────────────────

    fn check_function(&mut self, function: &FunctionDecl) -> Result<(), CompileError> {
        self.enter_scope();
        self.current_return_type = Some(function.return_type.clone());

        for param in &function.parameters {
            self.declare_variable(param.name.clone(), param.param_type.clone())?;
        }

        for stmt in &function.body {
            self.check_statement(stmt)?;
        }

        self.exit_scope();
        self.current_return_type = None;
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::LetDecl { name, var_type, initializer } => {
                if let Some(init) = initializer {
                    let actual = self.check_expression(init)?;
                    if &actual != var_type {
                        return Err(self.error(format!(
                            "type mismatch: variable '{}' declared as {:?} but initialiser has type {:?}",
                            name, var_type, actual
                        )));
                    }
                }
                self.declare_variable(name.clone(), var_type.clone())?;
            }

            Statement::Assignment { name, value } => {
                let declared = self.lookup_variable(name)?;
                let actual = self.check_expression(value)?;
                if declared != actual {
                    return Err(self.error(format!(
                        "cannot assign {:?} to '{}' which has type {:?}",
                        actual, name, declared
                    )));
                }
            }

            Statement::If { condition, then_branch, else_branch } => {
                self.expect_type(condition, &Type::Bool, "if condition")?;
                self.check_block(then_branch)?;
                if let Some(else_stmts) = else_branch {
                    self.check_block(else_stmts)?;
                }
            }

            Statement::While { condition, body } => {
                self.expect_type(condition, &Type::Bool, "while condition")?;
                self.loop_depth += 1;
                self.check_block(body)?;
                self.loop_depth -= 1;
            }

            Statement::For { var, start, end, body, .. } => {
                // Both bounds must be int.
                self.expect_type(start, &Type::Int, "for loop start bound")?;
                self.expect_type(end, &Type::Int, "for loop end bound")?;

                // The loop variable is scoped to the body.
                self.enter_scope();
                self.declare_variable(var.clone(), Type::Int)?;
                self.loop_depth += 1;
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.loop_depth -= 1;
                self.exit_scope();
            }

            Statement::Break | Statement::Continue => {
                if self.loop_depth == 0 {
                    let kw = if matches!(stmt, Statement::Break) { "break" } else { "continue" };
                    return Err(self.error(format!("'{}' used outside of a loop", kw)));
                }
            }

            Statement::Return { value } => {
                let expected = self.current_return_type.clone()
                    .ok_or_else(|| self.error("return statement outside of a function"))?;

                match (value, &expected) {
                    (Some(_), Type::Void) =>
                        return Err(self.error("cannot return a value from a void function")),
                    (None, Type::Void) => {}
                    (None, _) =>
                        return Err(self.error(format!("missing return value; expected {:?}", expected))),
                    (Some(expr), expected_type) => {
                        let actual = self.check_expression(expr)?;
                        if &actual != expected_type {
                            return Err(self.error(format!(
                                "return type mismatch: expected {:?}, got {:?}",
                                expected_type, actual
                            )));
                        }
                    }
                }
            }

            Statement::Print { expression } => { self.check_expression(expression)?; }
            Statement::Expression { expression } => { self.check_expression(expression)?; }
        }
        Ok(())
    }

    fn check_block(&mut self, stmts: &[Statement]) -> Result<(), CompileError> {
        self.enter_scope();
        for stmt in stmts {
            self.check_statement(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }

    // ── Expression type-checking ──────────────────────────────────────────────

    fn check_expression(&mut self, expr: &Expression) -> Result<Type, CompileError> {
        match expr {
            Expression::IntLiteral(_)    => Ok(Type::Int),
            Expression::BoolLiteral(_)   => Ok(Type::Bool),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::Variable(name)   => self.lookup_variable(name),

            Expression::Unary { operator, operand } => {
                let t = self.check_expression(operand)?;
                match operator {
                    UnaryOp::Negate => self.require(&t, &Type::Int, "negation operand must be int"),
                    UnaryOp::Not    => self.require(&t, &Type::Bool, "logical not operand must be bool"),
                }
            }

            Expression::Binary { left, operator, right } => {
                let lt = self.check_expression(left)?;
                let rt = self.check_expression(right)?;
                self.check_binary_op(operator, &lt, &rt)
            }

            Expression::Call { function, arguments } => {
                // Look up the signature without cloning the whole map entry.
                let sig = self.functions.get(function)
                    .ok_or_else(|| self.error(format!("undefined function '{}'", function)))?;

                let (param_types, return_type) = (sig.param_types.clone(), sig.return_type.clone());

                if arguments.len() != param_types.len() {
                    return Err(self.error(format!(
                        "'{}' expects {} argument(s), got {}",
                        function, param_types.len(), arguments.len()
                    )));
                }

                for (i, (arg, expected)) in arguments.iter().zip(param_types.iter()).enumerate() {
                    let actual = self.check_expression(arg)?;
                    if &actual != expected {
                        return Err(self.error(format!(
                            "argument {} of '{}': expected {:?}, got {:?}",
                            i + 1, function, expected, actual
                        )));
                    }
                }

                Ok(return_type)
            }
        }
    }

    fn check_binary_op(&self, op: &BinaryOp, lt: &Type, rt: &Type) -> Result<Type, CompileError> {
        match op {
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply |
            BinaryOp::Divide | BinaryOp::Modulo => {
                if lt != &Type::Int || rt != &Type::Int {
                    return Err(self.error(format!(
                        "arithmetic operators require int operands, got {:?} and {:?}", lt, rt
                    )));
                }
                Ok(Type::Int)
            }
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if lt != rt {
                    return Err(self.error(format!(
                        "equality operators require matching types, got {:?} and {:?}", lt, rt
                    )));
                }
                Ok(Type::Bool)
            }
            BinaryOp::Less | BinaryOp::LessEqual |
            BinaryOp::Greater | BinaryOp::GreaterEqual => {
                if lt != &Type::Int || rt != &Type::Int {
                    return Err(self.error(format!(
                        "comparison operators require int operands, got {:?} and {:?}", lt, rt
                    )));
                }
                Ok(Type::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if lt != &Type::Bool || rt != &Type::Bool {
                    return Err(self.error(format!(
                        "logical operators require bool operands, got {:?} and {:?}", lt, rt
                    )));
                }
                Ok(Type::Bool)
            }
        }
    }

    // ── Symbol-table helpers ──────────────────────────────────────────────────

    fn declare_variable(&mut self, name: String, ty: Type) -> Result<(), CompileError> {
        let scope = self.symbol_table.last_mut().unwrap();
        if scope.contains_key(&name) {
            return Err(self.error(format!("'{}' is already declared in this scope", name)));
        }
        scope.insert(name, ty);
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Result<Type, CompileError> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty.clone());
            }
        }
        Err(self.error(format!("undefined variable '{}'", name)))
    }

    fn enter_scope(&mut self) { self.symbol_table.push(HashMap::new()); }
    fn exit_scope(&mut self)  { self.symbol_table.pop(); }

    // ── Error construction ────────────────────────────────────────────────────

    fn error(&self, msg: impl Into<String>) -> CompileError {
        // Semantic errors don't have a token handy; use 0:0 as a sentinel.
        CompileError::new(CompilePhase::Semantic, msg, 0, 0)
    }

    /// Checks that `expr` has `expected` type; emits a contextualised error otherwise.
    fn expect_type(&mut self, expr: &Expression, expected: &Type, ctx: &str) -> Result<(), CompileError> {
        let actual = self.check_expression(expr)?;
        if &actual != expected {
            return Err(self.error(format!("{} must be {:?}, got {:?}", ctx, expected, actual)));
        }
        Ok(())
    }

    /// Returns `Ok(ty.clone())` when `ty == expected`, otherwise a compile error.
    fn require(&self, ty: &Type, expected: &Type, msg: &str) -> Result<Type, CompileError> {
        if ty == expected {
            Ok(ty.clone())
        } else {
            Err(self.error(format!("{}, got {:?}", msg, ty)))
        }
    }
}