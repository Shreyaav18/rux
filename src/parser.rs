use crate::ast::*;
use crate::token::{CompileError, CompilePhase, Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, CompileError> {
        let mut functions = Vec::new();
        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    // ── Declarations ─────────────────────────────────────────────────────────

    fn parse_function(&mut self) -> Result<FunctionDecl, CompileError> {
        self.consume(TokenType::Fn, "expected 'fn'")?;
        let name = self.consume(TokenType::Identifier, "expected function name")?.lexeme.clone();

        self.consume(TokenType::LeftParen, "expected '(' after function name")?;
        let parameters = self.parse_parameter_list()?;
        self.consume(TokenType::RightParen, "expected ')' after parameters")?;

        let return_type = if self.match_token(TokenType::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;

        Ok(FunctionDecl { name, parameters, return_type, body })
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, CompileError> {
        let mut params = Vec::new();
        if self.check(&TokenType::RightParen) {
            return Ok(params);
        }
        loop {
            let name = self.consume(TokenType::Identifier, "expected parameter name")?.lexeme.clone();
            self.consume(TokenType::Colon, "expected ':' after parameter name")?;
            let param_type = self.parse_type()?;
            params.push(Parameter { name, param_type });
            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, CompileError> {
        let token = self.advance().clone();
        match token.token_type {
            TokenType::Int  => Ok(Type::Int),
            TokenType::Bool => Ok(Type::Bool),
            TokenType::String => Ok(Type::String),
            _ => Err(CompileError::at(
                CompilePhase::Parser,
                format!("expected type, got '{}'", token.lexeme),
                &token,
            )),
        }
    }

    // ── Statements ───────────────────────────────────────────────────────────

    /// Parses `{ ... }` including the braces.
    fn parse_block(&mut self) -> Result<Vec<Statement>, CompileError> {
        self.consume(TokenType::LeftBrace, "expected '{'")?;
        let mut stmts = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.consume(TokenType::RightBrace, "expected '}'")?;
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        if self.match_token(TokenType::Let)      { return self.parse_let_statement(); }
        if self.match_token(TokenType::If)       { return self.parse_if_statement(); }
        if self.match_token(TokenType::While)    { return self.parse_while_statement(); }
        if self.match_token(TokenType::For)      { return self.parse_for_statement(); }
        if self.match_token(TokenType::Return)   { return self.parse_return_statement(); }
        if self.match_token(TokenType::Print)    { return self.parse_print_statement(); }
        if self.match_token(TokenType::Break)    { return self.parse_break_statement(); }
        if self.match_token(TokenType::Continue) { return self.parse_continue_statement(); }

        // Assignment: `ident =`
        if self.check(&TokenType::Identifier)
            && self.peek_ahead(1).token_type == TokenType::Equal
        {
            return self.parse_assignment_statement();
        }

        self.parse_expression_statement()
    }

    fn parse_let_statement(&mut self) -> Result<Statement, CompileError> {
        let name = self.consume(TokenType::Identifier, "expected variable name")?.lexeme.clone();
        self.consume(TokenType::Colon, "expected ':' after variable name")?;
        let var_type = self.parse_type()?;

        let initializer = if self.match_token(TokenType::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "expected ';' after variable declaration")?;
        Ok(Statement::LetDecl { name, var_type, initializer })
    }

    fn parse_assignment_statement(&mut self) -> Result<Statement, CompileError> {
        let name = self.consume(TokenType::Identifier, "expected variable name")?.lexeme.clone();
        self.consume(TokenType::Equal, "expected '='")?;
        let value = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after assignment")?;
        Ok(Statement::Assignment { name, value })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, CompileError> {
        self.consume(TokenType::LeftParen, "expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "expected ')' after condition")?;

        let then_branch = self.parse_block()?;

        // Support `else if` chains as well as plain `else`.
        let else_branch = if self.match_token(TokenType::Else) {
            if self.check(&TokenType::If) {
                // `else if` — consume the `if` and recurse; wrap the result in a vec
                self.advance(); // consume `if`
                Some(vec![self.parse_if_statement()?])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Statement::If { condition, then_branch, else_branch })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, CompileError> {
        self.consume(TokenType::LeftParen, "expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "expected ')' after condition")?;
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, CompileError> {
        let var = self.consume(TokenType::Identifier, "expected loop variable after 'for'")?.lexeme.clone();
        self.consume(TokenType::In, "expected 'in' after loop variable")?;
        let start = self.parse_expression()?;

        let inclusive = if self.match_token(TokenType::DotDotEq) {
            true
        } else {
            self.consume(TokenType::DotDot, "expected '..' or '..=' in range")?;
            false
        };

        let end = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For { var, start, end, inclusive, body })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, CompileError> {
        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::Semicolon, "expected ';' after return")?;
        Ok(Statement::Return { value })
    }

    fn parse_print_statement(&mut self) -> Result<Statement, CompileError> {
        self.consume(TokenType::LeftParen, "expected '(' after 'print'")?;
        let expression = self.parse_expression()?;
        self.consume(TokenType::RightParen, "expected ')' after expression")?;
        self.consume(TokenType::Semicolon, "expected ';' after print")?;
        Ok(Statement::Print { expression })
    }

    fn parse_break_statement(&mut self) -> Result<Statement, CompileError> {
        let tok = self.previous().clone();
        self.consume(TokenType::Semicolon, "expected ';' after 'break'")
            .map_err(|_| CompileError::at(CompilePhase::Parser, "expected ';' after 'break'", &tok))?;
        Ok(Statement::Break)
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, CompileError> {
        let tok = self.previous().clone();
        self.consume(TokenType::Semicolon, "expected ';' after 'continue'")
            .map_err(|_| CompileError::at(CompilePhase::Parser, "expected ';' after 'continue'", &tok))?;
        Ok(Statement::Continue)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, CompileError> {
        let expression = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after expression")?;
        Ok(Statement::Expression { expression })
    }

    // ── Expressions (Pratt-style precedence climbing) ─────────────────────────

    fn parse_expression(&mut self) -> Result<Expression, CompileError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_and()?;
        while self.match_token(TokenType::Or) {
            let right = self.parse_and()?;
            expr = Expression::Binary { left: Box::new(expr), operator: BinaryOp::Or, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_equality()?;
        while self.match_token(TokenType::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary { left: Box::new(expr), operator: BinaryOp::And, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_comparison()?;
        while let Some(op) = self.match_any(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let right = self.parse_comparison()?;
            expr = Expression::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_term()?;
        while let Some(op) = self.match_any(&[
            TokenType::Less, TokenType::LessEqual,
            TokenType::Greater, TokenType::GreaterEqual,
        ]) {
            let right = self.parse_term()?;
            expr = Expression::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_factor()?;
        while let Some(op) = self.match_any(&[TokenType::Plus, TokenType::Minus]) {
            let right = self.parse_factor()?;
            expr = Expression::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, CompileError> {
        let mut expr = self.parse_unary()?;
        while let Some(op) = self.match_any(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let right = self.parse_unary()?;
            expr = Expression::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, CompileError> {
        if let Some(op) = self.match_any(&[TokenType::Minus, TokenType::Bang]) {
            let unary_op = match op {
                BinaryOp::Subtract => UnaryOp::Negate,
                // Bang matched → Not; Minus → Negate — handle via token directly
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary { operator: unary_op, operand: Box::new(operand) });
        }

        // Re-implement without match_any so we can map directly to UnaryOp
        if self.match_token(TokenType::Minus) {
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary { operator: UnaryOp::Negate, operand: Box::new(operand) });
        }
        if self.match_token(TokenType::Bang) {
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary { operator: UnaryOp::Not, operand: Box::new(operand) });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, CompileError> {
        let expr = self.parse_primary()?;

        if self.match_token(TokenType::LeftParen) {
            let name = match expr {
                Expression::Variable(ref name) => name.clone(),
                _ => {
                    let tok = self.previous().clone();
                    return Err(CompileError::at(
                        CompilePhase::Parser,
                        "only named functions can be called",
                        &tok,
                    ));
                }
            };

            let mut arguments = Vec::new();
            if !self.check(&TokenType::RightParen) {
                loop {
                    arguments.push(self.parse_expression()?);
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume(TokenType::RightParen, "expected ')' after arguments")?;
            return Ok(Expression::Call { function: name, arguments });
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, CompileError> {
        if self.match_token(TokenType::True)  { return Ok(Expression::BoolLiteral(true)); }
        if self.match_token(TokenType::False) { return Ok(Expression::BoolLiteral(false)); }

        if self.match_token(TokenType::IntLiteral) {
            let tok = self.previous().clone();
            let value = tok.lexeme.parse::<i64>().map_err(|_| {
                CompileError::at(CompilePhase::Parser, format!("invalid integer literal '{}'", tok.lexeme), &tok)
            })?;
            return Ok(Expression::IntLiteral(value));
        }

        if self.match_token(TokenType::StringLiteral) {
            return Ok(Expression::StringLiteral(self.previous().lexeme.clone()));
        }

        if self.match_token(TokenType::Identifier) {
            return Ok(Expression::Variable(self.previous().lexeme.clone()));
        }

        if self.match_token(TokenType::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(TokenType::RightParen, "expected ')' after grouped expression")?;
            return Ok(expr);
        }

        let tok = self.peek().clone();
        Err(CompileError::at(
            CompilePhase::Parser,
            format!("unexpected token '{}'", tok.lexeme),
            &tok,
        ))
    }

    // ── Primitive helpers ─────────────────────────────────────────────────────

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(&token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Tries each type in order; on first match advances and returns the
    /// corresponding `BinaryOp`. Returns `None` if nothing matched.
    fn match_any(&mut self, types: &[TokenType]) -> Option<BinaryOp> {
        for tt in types {
            if self.check(tt) {
                self.advance();
                return Some(token_type_to_binary_op(tt));
            }
        }
        None
    }

    fn check(&self, token_type: &TokenType) -> bool {
        !self.is_at_end() && &self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Returns a token `offset` positions ahead; clamps to the last token (EOF).
    fn peek_ahead(&self, offset: usize) -> &Token {
        let idx = (self.current + offset).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, CompileError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            let tok = self.peek().clone();
            Err(CompileError::at(CompilePhase::Parser, message, &tok))
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn token_type_to_binary_op(tt: &TokenType) -> BinaryOp {
    match tt {
        TokenType::Plus         => BinaryOp::Add,
        TokenType::Minus        => BinaryOp::Subtract,
        TokenType::Star         => BinaryOp::Multiply,
        TokenType::Slash        => BinaryOp::Divide,
        TokenType::Percent      => BinaryOp::Modulo,
        TokenType::EqualEqual   => BinaryOp::Equal,
        TokenType::BangEqual    => BinaryOp::NotEqual,
        TokenType::Less         => BinaryOp::Less,
        TokenType::LessEqual    => BinaryOp::LessEqual,
        TokenType::Greater      => BinaryOp::Greater,
        TokenType::GreaterEqual => BinaryOp::GreaterEqual,
        TokenType::And          => BinaryOp::And,
        TokenType::Or           => BinaryOp::Or,
        _ => unreachable!("token type {:?} has no BinaryOp mapping", tt),
    }
}