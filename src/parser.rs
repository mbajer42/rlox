use crate::error::{LoxError, Result};
use crate::statement::{Expr, Stmt};
use crate::token::{Token, TokenType};

use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

// TODO find a better solution
static NEXT_EXPRESSION_ID: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    NEXT_EXPRESSION_ID.fetch_add(1, Ordering::Relaxed)
}

struct Parser<'a> {
    token_iter: std::iter::Peekable<std::slice::Iter<'a, Token<'a>>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token<'a>>) -> Self {
        Self {
            token_iter: tokens.iter().peekable(),
        }
    }

    fn statement(&mut self) -> Result<Stmt> {
        if let Some(token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::Print => self.print_statement(),
                TokenType::Var => self.var_declaration(),
                TokenType::LeftBrace => self.block(),
                TokenType::If => self.if_statement(),
                TokenType::While => self.while_statement(),
                TokenType::For => self.for_statement(),
                TokenType::Fun => {
                    self.token_iter.next();
                    self.function()
                }
                TokenType::Class => self.class(),
                TokenType::Return => self.return_statement(),
                _ => self.expression_statement(),
            }
        } else {
            unreachable!()
        }
    }

    fn class(&mut self) -> Result<Stmt> {
        self.consume(TokenType::Class, "Classes begin with 'class'")?;
        let name = self.identifier_name("class")?;

        let superclass = if self.matches(&[TokenType::Less]) {
            self.token_iter.next();
            let superclass_identifier = self.identifier_name("class")?;
            Some(Box::new(Expr::Variable {
                id: next_id(),
                name: superclass_identifier.to_string(),
            }))
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body".into())?;

        let mut methods = vec![];
        while !self.matches(&[TokenType::RightBrace]) {
            methods.push(self.function()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body".into())?;

        Ok(Stmt::Class {
            name: name.to_string(),
            superclass,
            methods: Box::new(methods),
        })
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::Return, "Return statements begin with 'return'")?;
        let value = if self.matches(&[TokenType::Semicolon]) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Stmt::Return { value })
    }

    fn function(&mut self) -> Result<Stmt> {
        let name = self.identifier_name("function")?;
        self.consume(
            TokenType::LeftParen,
            "Expect '(' after function name".into(),
        )?;

        let mut parameters = vec![];
        while !self.matches(&[TokenType::RightParen]) {
            let parameter_name = self.identifier_name("parameter")?;
            parameters.push(parameter_name.to_string());
            if self.matches(&[TokenType::Comma]) {
                self.token_iter.next();
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        let statements = if let Stmt::Block { statements } = self.block()? {
            statements
        } else {
            return Err(LoxError::ParserError(None, "Expect function body".into()));
        };

        Ok(Stmt::Function {
            name: name.to_string(),
            parameters: Rc::new(parameters),
            body: Rc::from(statements),
        })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::While, "While loops begin with 'while'.")?;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;

        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::For, "For loops begin with 'for'.")?;
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.matches(&[TokenType::Semicolon]) {
            self.token_iter.next();
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.matches(&[TokenType::Semicolon]) {
            Expr::Boolean(true)
        } else {
            self.expression()?
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.matches(&[TokenType::RightParen]) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if increment.is_some() {
            body = Stmt::Block {
                statements: Box::new(vec![
                    body,
                    Stmt::Expression {
                        expression: increment.unwrap(),
                    },
                ]),
            };
        };
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if initializer.is_some() {
            body = Stmt::Block {
                statements: Box::new(vec![initializer.unwrap(), body]),
            };
        };

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::If, "If statements begin with 'if'.")?;
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;

        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.matches(&[TokenType::Else]) {
            self.token_iter.next();
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftBrace, "Blocks begin with '{'.")?;
        let mut statements = Box::new(vec![]);

        while !self.matches(&[TokenType::RightBrace]) {
            statements.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block { statements })
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::Print, "Print statements begin with 'print'.")?;
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression })
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        self.consume(TokenType::Var, "Var declarations begin with 'var'.")?;
        if let Some(token) = self.token_iter.next() {
            match token.token_type {
                TokenType::Identifier => {
                    let name = token.lexeme;
                    let initializer = if self.matches(&[TokenType::Equal]) {
                        self.token_iter.next();
                        Some(self.expression()?)
                    } else {
                        None
                    };
                    self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
                    Ok(Stmt::Var {
                        name: name.to_string(),
                        initializer,
                    })
                }
                _ => Err(LoxError::ParserError(
                    Some(token.line),
                    "Expect variable name after 'var'.".into(),
                )),
            }
        } else {
            Err(LoxError::ParserError(
                None,
                "Expect variable name after 'var'.".into(),
            ))
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.matches(&[TokenType::Equal]) {
            self.token_iter.next();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { id: _, name } => Ok(Expr::Assign {
                    id: next_id(),
                    name,
                    value: Box::new(value),
                }),
                Expr::Get { object, name } => Ok(Expr::Set {
                    object,
                    name,
                    value: Rc::new(value),
                }),
                _ => Err(LoxError::ParserError(
                    None,
                    "Invalid assignment target".into(),
                )),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.matches(&[TokenType::Or]) {
            self.token_iter.next();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: TokenType::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            self.token_iter.next();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: TokenType::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while let Some(&token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::BangEqual | TokenType::LessEqual => {
                    self.token_iter.next();
                    let right = self.addition()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        token_type: token.token_type.clone(),
                        right: Box::new(right),
                    };
                }
                _ => break,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.addition()?;

        while let Some(&token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::Less
                | TokenType::LessEqual => {
                    self.token_iter.next();
                    let right = self.addition()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        token_type: token.token_type.clone(),
                        right: Box::new(right),
                    };
                }
                _ => break,
            };
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr> {
        let mut expr = self.multiplication()?;

        while let Some(&token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::Minus | TokenType::Plus => {
                    self.token_iter.next();
                    let right = self.multiplication()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        token_type: token.token_type.clone(),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while let Some(&token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::Slash | TokenType::Star => {
                    self.token_iter.next();
                    let right = self.unary()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        token_type: token.token_type.clone(),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(&token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::Bang | TokenType::Minus => {
                    self.token_iter.next();
                    let right = self.unary()?;
                    Ok(Expr::Unary {
                        token_type: token.token_type.clone(),
                        right: Box::new(right),
                    })
                }
                _ => self.call(),
            }
        } else {
            unreachable!();
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                self.token_iter.next();
                expr = self.finish_call(expr)?;
            } else if self.matches(&[TokenType::Dot]) {
                self.token_iter.next();

                let token = self.token_iter.next();
                if let Some(token) = token {
                    match &token.token_type {
                        TokenType::Identifier => {
                            expr = Expr::Get {
                                object: Box::new(expr),
                                name: token.lexeme.to_string(),
                            };
                        }
                        _ => {
                            return Err(LoxError::ParserError(
                                Some(token.line),
                                "Expect property name after '.'.".into(),
                            ))
                        }
                    }
                } else {
                    return Err(LoxError::ParserError(
                        None,
                        "Expect property name after '.'.".into(),
                    ));
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = vec![];
        if !self.matches(&[TokenType::RightParen]) {
            arguments.push(self.expression()?);
            while self.matches(&[TokenType::Comma]) {
                self.token_iter.next();
                arguments.push(self.expression()?);
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments: Box::new(arguments),
        })
    }

    fn primary(&mut self) -> Result<Expr> {
        if let Some(token) = self.token_iter.next() {
            match token.token_type {
                TokenType::False => Ok(Expr::Boolean(false)),
                TokenType::True => Ok(Expr::Boolean(true)),
                TokenType::Nil => Ok(Expr::Nil),
                TokenType::Number(num) => Ok(Expr::Number(num)),
                TokenType::String(ref string) => Ok(Expr::String(string.to_string())),
                TokenType::LeftParen => {
                    let expr = self.expression()?;
                    if let Some(token) = self.token_iter.next() {
                        if &token.token_type == &TokenType::RightParen {
                            Ok(Expr::Grouping {
                                expression: Box::new(expr),
                            })
                        } else {
                            Err(LoxError::ParserError(
                                Some(token.line),
                                format!("Expected ')' but got '{}'", &token.lexeme).into(),
                            ))
                        }
                    } else {
                        Parser::expected_expression(None)
                    }
                }
                TokenType::Identifier => Ok(Expr::Variable {
                    id: next_id(),
                    name: token.lexeme.to_string(),
                }),
                TokenType::Super => {
                    self.consume(TokenType::Dot, "Expect '.' after super.")?;
                    let token = self.token_iter.next();
                    let method = if let Some(token) = token {
                        if token.token_type != TokenType::Identifier {
                            return Err(LoxError::ParserError(
                                None,
                                "Expect superclass method name.".into(),
                            ));
                        } else {
                            token.lexeme
                        }
                    } else {
                        return Err(LoxError::ParserError(
                            None,
                            "Expect superclass method name.".into(),
                        ));
                    };
                    Ok(Expr::Super {
                        id: next_id(),
                        keyword: "super",
                        method: method.to_string(),
                    })
                }
                TokenType::This => Ok(Expr::This {
                    id: next_id(),
                    keyword: "this",
                }),
                _ => Parser::expected_expression(None),
            }
        } else {
            Parser::expected_expression(None)
        }
    }

    fn identifier_name(&mut self, kind: &'static str) -> Result<&'a str> {
        if let Some(token) = self.token_iter.next() {
            match token.token_type {
                TokenType::Identifier => Ok(token.lexeme),
                _ => {
                    return Err(LoxError::ParserError(
                        Some(token.line),
                        format!("Expect {} name", kind).into(),
                    ));
                }
            }
        } else {
            return Err(LoxError::ParserError(
                None,
                format!("Expect {} name.", kind).into(),
            ));
        }
    }

    fn matches(&mut self, token_types: &[TokenType]) -> bool {
        self.token_iter
            .peek()
            .map(|token| token_types.contains(&token.token_type))
            .unwrap_or(false)
    }

    fn consume(&mut self, token_type: TokenType, error_message: &'static str) -> Result<()> {
        if let Some(token) = self.token_iter.next() {
            if token.token_type == token_type {
                Ok(())
            } else {
                Err(LoxError::ParserError(
                    Some(token.line),
                    error_message.into(),
                ))
            }
        } else {
            Err(LoxError::ParserError(None, error_message.into()))
        }
    }

    fn expected_expression(line: Option<u32>) -> Result<Expr> {
        Err(LoxError::ParserError(
            line,
            "Unexpected end of file, expected expression.".into(),
        ))
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.token_iter.peek() {
            None
            | Some(Token {
                token_type: TokenType::Eof,
                ..
            }) => None,
            _ => Some(self.statement()),
        }
    }
}

pub fn parse<'a>(tokens: &'a Vec<Token<'a>>) -> (Vec<Stmt>, Vec<LoxError>) {
    let parser = Parser::new(tokens);
    let (expressions, errors): (Vec<_>, Vec<_>) = parser.partition(Result::is_ok);
    let expressions = expressions.into_iter().map(Result::unwrap).collect();
    let errors = errors.into_iter().map(Result::unwrap_err).collect();

    (expressions, errors)
}

#[cfg(test)]
mod tests {

    use super::parse;
    use super::{Expr, Stmt};
    use crate::lexer;
    use crate::token::TokenType;

    #[test]
    fn simple_mathematical_expression() {
        let source = "(3 + 4) * 6;";
        let (tokens, _) = lexer::lex(source);
        let (statements, errors) = parse(&tokens);
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);

        let expected_expression = "Binary { left: \
            Grouping { expression: Binary { left: Number(3.0), token_type: Plus, right: Number(4.0) } }, \
            token_type: Star, \
            right: Number(6.0) }";

        match &statements[0] {
            Stmt::Expression { expression } => {
                assert_eq!(format!("{:?}", expression), expected_expression)
            }
            something_else => panic!("Expected expression statement, got '{:?}'", something_else),
        }
    }

    #[test]
    fn var_declaration() {
        let source = "var answer = 42;";
        let (tokens, _) = lexer::lex(source);
        let (mut statements, errors) = parse(&tokens);
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);

        match statements.remove(0) {
            Stmt::Var { name, initializer } => {
                assert_eq!(name, "answer");
                assert_eq!(initializer.unwrap(), Expr::Number(42.0));
            }
            _ => panic!("Expected to be of type Stmt::Var"),
        }
    }

    #[test]
    fn or_operator() {
        let source = "true or false;";
        let (tokens, _) = lexer::lex(source);
        let (statements, errors) = parse(&tokens);
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            &Stmt::Expression { ref expression } => {
                assert_eq!(
                    expression,
                    &Expr::Logical {
                        left: Box::new(Expr::Boolean(true)),
                        operator: TokenType::Or,
                        right: Box::new(Expr::Boolean(false))
                    }
                );
            }
            _ => panic!("Expected to be of type Stmt::Expression"),
        }
    }

    #[test]
    fn function() {
        let source = r#"
            fun add(a, b) {
                print a + b;
            }
        "#;
        let (tokens, _) = lexer::lex(source);
        let (statements, errors) = parse(&tokens);
        assert_eq!(errors.len(), 0);
        assert_eq!(statements.len(), 1);
    }
}
