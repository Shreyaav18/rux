#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    String,
    Void,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub enum Statement {
    LetDecl {
        name: String,
        var_type: Type,
        initializer: Option<Expression>,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Return {
        value: Option<Expression>,
    },
    Print {
        expression: Expression,
    },
    Expression {
        expression: Expression,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntLiteral(i64),
    BoolLiteral(bool),
    StringLiteral(String),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}