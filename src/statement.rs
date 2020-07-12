use crate::token::TokenType;

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    // literal values
    Number(f64),
    String(&'a str),
    Boolean(bool),
    Nil,
    // compound expressions
    Binary {
        left: Box<Expr<'a>>,
        token_type: TokenType<'a>,
        right: Box<Expr<'a>>,
    },
    Call {
        callee: Box<Expr<'a>>,
        arguments: Box<Vec<Expr<'a>>>,
    },
    Grouping {
        expression: Box<Expr<'a>>,
    },
    Unary {
        token_type: TokenType<'a>,
        right: Box<Expr<'a>>,
    },
    Logical {
        left: Box<Expr<'a>>,
        operator: TokenType<'a>,
        right: Box<Expr<'a>>,
    },
    // assignments
    Variable {
        name: &'a str,
    },
    Assign {
        name: &'a str,
        value: Box<Expr<'a>>,
    },
}

#[derive(Debug)]
pub enum Stmt<'a> {
    Expression {
        expression: Expr<'a>,
    },
    Print {
        expression: Expr<'a>,
    },
    Var {
        name: &'a str,
        initializer: Option<Expr<'a>>,
    },
    Block {
        statements: Box<Vec<Stmt<'a>>>,
    },
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Stmt<'a>>,
    },
}
