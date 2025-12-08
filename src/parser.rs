use crate::ast::*;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();

        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, String> {
    self.consume(TokenType::Fn, "Expected 'fn'")?;
    
    let func_name = self.consume(TokenType::Identifier, "Expected function name")?.lexeme.clone();

    self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

    let mut parameters = Vec::new();
    if !self.check(TokenType::RightParen) {
        loop {
            let param_name = self.consume(TokenType::Identifier, "Expected parameter name")?.lexeme.clone();
            self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
            let param_type = self.parse_type()?;

            parameters.push(Parameter {
                name: param_name,
                param_type,
            });

            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
    }

    self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

    let return_type = if self.match_token(TokenType::Arrow) {
        self.parse_type()?
    } else {
        Type::Void
    };

    self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;

    let mut body = Vec::new();
    while !self.check(TokenType::RightBrace) && !self.is_at_end() {
        body.push(self.parse_statement()?);
    }

    self.consume(TokenType::RightBrace, "Expected '}' after function body")?;

    Ok(FunctionDecl {
        name: func_name,
        parameters,
        return_type,
        body,
    })
}

    fn parse_type(&mut self) -> Result<Type, String> {
        let token = self.advance();
        match token.token_type {
            TokenType::Int => Ok(Type::Int),
            TokenType::Bool => Ok(Type::Bool),
            TokenType::String => Ok(Type::String),
            _ => Err(format!("Expected type at line {}, column {}", token.line, token.column)),
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.match_token(TokenType::Let) {
            self.parse_let_statement()
        } else if self.match_token(TokenType::If) {
            self.parse_if_statement()
        } else if self.match_token(TokenType::While) {
            self.parse_while_statement()
        } else if self.match_token(TokenType::Return) {
            self.parse_return_statement()   
        } else if self.match_token(TokenType::Print) {
            self.parse_print_statement()
        } else if self.check(TokenType::Identifier) && self.peek_ahead(1).token_type == TokenType::Equal {
            self.parse_assignment_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let var_name = self.consume(TokenType::Identifier, "Expected variable name")?.lexeme.clone();
        self.consume(TokenType::Colon, "Expected ':' after variable name")?;
        let var_type = self.parse_type()?;

        let initializer = if self.match_token(TokenType::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;

        Ok(Statement::LetDecl {
            name: var_name,
            var_type,
            initializer,
        })
    }

    fn parse_assignment_statement(&mut self) -> Result<Statement, String> {
        let var_name = self.consume(TokenType::Identifier, "Expected variable name")?.lexeme.clone();
        self.consume(TokenType::Equal, "Expected '='")?;
        let value = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after assignment")?;

        Ok(Statement::Assignment {
            name: var_name,
            value,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;

        self.consume(TokenType::LeftBrace, "Expected '{' after if condition")?;
        let mut then_branch = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            then_branch.push(self.parse_statement()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after then branch")?;

        let else_branch = if self.match_token(TokenType::Else) {
            self.consume(TokenType::LeftBrace, "Expected '{' after 'else'")?;
            let mut else_stmts = Vec::new();
            while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                else_stmts.push(self.parse_statement()?);
            }
            self.consume(TokenType::RightBrace, "Expected '}' after else branch")?;
            Some(else_stmts)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;

        self.consume(TokenType::LeftBrace, "Expected '{' after while condition")?;
        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after while body")?;

        Ok(Statement::While { condition, body })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return statement")?;

        Ok(Statement::Return { value })
    }

    fn parse_print_statement(&mut self) -> Result<Statement, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'print'")?;
        let expression = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after expression")?;
        self.consume(TokenType::Semicolon, "Expected ';' after print statement")?;
        Ok(Statement::Print { expression })
    }
    
    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let expression = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::Expression { expression })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and()?;

        while self.match_token(TokenType::Or) {
            let right = self.parse_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;

        while self.match_token(TokenType::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;

        while self.match_tokens(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = match self.previous().token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_term()?;

        while self.match_tokens(&[TokenType::Less, TokenType::LessEqual, TokenType::Greater, TokenType::GreaterEqual]) {
            let operator = match self.previous().token_type {
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_factor()?;

        while self.match_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = match self.previous().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;

        while self.match_tokens(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let operator = match self.previous().token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_tokens(&[TokenType::Minus, TokenType::Bang]) {
            let operator = match self.previous().token_type {
                TokenType::Minus => UnaryOp::Negate,
                TokenType::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator,
                operand: Box::new(operand),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        if self.match_token(TokenType::LeftParen) {
            if let Expression::Variable(name) = expr {
                let mut arguments = Vec::new();

                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

                expr = Expression::Call {
                    function: name,
                    arguments,
                };
            } else {
                return Err("Can only call functions".to_string());
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        if self.match_token(TokenType::True) {
            return Ok(Expression::BoolLiteral(true));
        }

        if self.match_token(TokenType::False) {
            return Ok(Expression::BoolLiteral(false));
        }

        if self.match_token(TokenType::IntLiteral) {
            let value = self.previous().lexeme.parse::<i64>()
                .map_err(|_| "Invalid integer literal".to_string())?;
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
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        Err(format!("Unexpected token: {:?}", self.peek()))
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
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

    fn peek_ahead(&self, offset: usize) -> &Token {
        let index = self.current + offset;
        if index < self.tokens.len() {
            &self.tokens[index]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(format!("{} at line {}, column {}", message, token.line, token.column))
        }
    }
}