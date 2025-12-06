use crate::token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token::new(TokenType::Eof, String::new(), self.line, self.column));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.current_char();

        let token = match ch {
            '+' => {
                self.advance();
                Token::new(TokenType::Plus, "+".to_string(), start_line, start_column)
            }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Token::new(TokenType::Arrow, "->".to_string(), start_line, start_column)
                } else {
                    Token::new(TokenType::Minus, "-".to_string(), start_line, start_column)
                }
            }
            '*' => {
                self.advance();
                Token::new(TokenType::Star, "*".to_string(), start_line, start_column)
            }
            '/' => {
                self.advance();
                Token::new(TokenType::Slash, "/".to_string(), start_line, start_column)
            }
            '%' => {
                self.advance();
                Token::new(TokenType::Percent, "%".to_string(), start_line, start_column)
            }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenType::EqualEqual, "==".to_string(), start_line, start_column)
                } else {
                    Token::new(TokenType::Equal, "=".to_string(), start_line, start_column)
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenType::BangEqual, "!=".to_string(), start_line, start_column)
                } else {
                    Token::new(TokenType::Bang, "!".to_string(), start_line, start_column)
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenType::LessEqual, "<=".to_string(), start_line, start_column)
                } else {
                    Token::new(TokenType::Less, "<".to_string(), start_line, start_column)
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::new(TokenType::GreaterEqual, ">=".to_string(), start_line, start_column)
                } else {
                    Token::new(TokenType::Greater, ">".to_string(), start_line, start_column)
                }
            }
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    Token::new(TokenType::And, "&&".to_string(), start_line, start_column)
                } else {
                    return Err(format!("Unexpected character '&' at line {}, column {}", start_line, start_column));
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    Token::new(TokenType::Or, "||".to_string(), start_line, start_column)
                } else {
                    return Err(format!("Unexpected character '|' at line {}, column {}", start_line, start_column));
                }
            }
            '(' => {
                self.advance();
                Token::new(TokenType::LeftParen, "(".to_string(), start_line, start_column)
            }
            ')' => {
                self.advance();
                Token::new(TokenType::RightParen, ")".to_string(), start_line, start_column)
            }
            '{' => {
                self.advance();
                Token::new(TokenType::LeftBrace, "{".to_string(), start_line, start_column)
            }
            '}' => {
                self.advance();
                Token::new(TokenType::RightBrace, "}".to_string(), start_line, start_column)
            }
            ',' => {
                self.advance();
                Token::new(TokenType::Comma, ",".to_string(), start_line, start_column)
            }
            ';' => {
                self.advance();
                Token::new(TokenType::Semicolon, ";".to_string(), start_line, start_column)
            }
            ':' => {
                self.advance();
                Token::new(TokenType::Colon, ":".to_string(), start_line, start_column)
            }
            '"' => self.read_string()?,
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ => return Err(format!("Unexpected character '{}' at line {}, column {}", ch, start_line, start_column)),
        };

        Ok(token)
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        self.advance();

        let mut value = String::new();
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(format!("Unterminated string at line {}, column {}", start_line, start_column));
                }
                match self.current_char() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    _ => {
                        value.push('\\');
                        value.push(self.current_char());
                    }
                }
                self.advance();
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err(format!("Unterminated string at line {}, column {}", start_line, start_column));
        }

        self.advance();
        Ok(Token::new(TokenType::StringLiteral, value, start_line, start_column))
    }

    fn read_number(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            value.push(self.current_char());
            self.advance();
        }

        Token::new(TokenType::IntLiteral, value, start_line, start_column)
    }

    fn read_identifier(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        while !self.is_at_end() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            value.push(self.current_char());
            self.advance();
        }

        let token_type = match value.as_str() {
            "int" => TokenType::Int,
            "bool" => TokenType::Bool,
            "string" => TokenType::String,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "let" => TokenType::Let,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "return" => TokenType::Return,
            _ => TokenType::Identifier,
        };

        Token::new(token_type, value, start_line, start_column)
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}