/// Structured compile error — replaces bare `String` errors throughout the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub phase: CompilePhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompilePhase {
    Lexer,
    Parser,
    Semantic,
    Codegen,
}

impl CompileError {
    pub fn new(phase: CompilePhase, message: impl Into<String>, line: usize, column: usize) -> Self {
        Self { message: message.into(), line, column, phase }
    }

    pub fn at(phase: CompilePhase, message: impl Into<String>, token: &Token) -> Self {
        Self::new(phase, message, token.line, token.column)
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}:{}: {}", self.phase, self.line, self.column, self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Primitive types
    Int,
    Bool,
    String,

    // Literals
    Identifier,
    IntLiteral,
    StringLiteral,

    // Keywords
    True,
    False,
    Let,
    Fn,
    If,
    Else,
    While,
    Break,
    Continue,
    Return,
    Print,

    // Arithmetic operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Comparison / logical
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Bang,
    And,
    Or,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Colon,
    Arrow,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: impl Into<String>, line: usize, column: usize) -> Self {
        Self { token_type, lexeme: lexeme.into(), line, column }
    }
}