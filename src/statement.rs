use crate::token::TokenType;

use std::rc::Rc;

pub type ExprId = u64;

#[derive(Debug, PartialEq)]
pub enum Expr {
    // literal values
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    // compound expressions
    Binary {
        left: Box<Expr>,
        token_type: TokenType,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Box<Vec<Expr>>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
    Set {
        object: Box<Expr>,
        name: String,
        value: Rc<Expr>,
    },
    Super {
        id: ExprId,
        keyword: &'static str,
        method: String,
    },
    This {
        id: ExprId,
        keyword: &'static str,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Unary {
        token_type: TokenType,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
    // assignments
    Variable {
        id: ExprId,
        name: String,
    },
    Assign {
        id: ExprId,
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    Block {
        statements: Box<Vec<Stmt>>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: String,
        parameters: Rc<Vec<String>>,
        body: Rc<Vec<Stmt>>,
    },
    Return {
        value: Option<Expr>,
    },
    Class {
        name: String,
        superclass: Option<Box<Expr>>,
        methods: Box<Vec<Stmt>>,
    },
}
