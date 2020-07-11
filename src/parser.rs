use crate::error::{LoxError, Result};
use crate::statement::{Expr, Stmt};
use crate::token::{Token, TokenType};

struct Parser<'a> {
    token_iter: std::iter::Peekable<std::slice::Iter<'a, Token<'a>>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token<'a>>) -> Self {
        Self {
            token_iter: tokens.iter().peekable(),
        }
    }

    fn statement(&mut self) -> Result<Stmt<'a>> {
        if let Some(token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::Print => {
                    self.token_iter.next();
                    self.print_statement()
                }
                TokenType::Var => {
                    self.token_iter.next();
                    self.var_declaration()
                }
                TokenType::LeftBrace => {
                    self.token_iter.next();
                    self.block()
                }
                _ => self.expression_statement(),
            }
        } else {
            unreachable!()
        }
    }

    fn block(&mut self) -> Result<Stmt<'a>> {
        let mut statements = Box::new(vec![]);

        while !self.matches(&[TokenType::RightBrace]) {
            statements.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block { statements })
    }

    fn print_statement(&mut self) -> Result<Stmt<'a>> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression })
    }

    fn var_declaration(&mut self) -> Result<Stmt<'a>> {
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
                    Ok(Stmt::Var { name, initializer })
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

    fn expression_statement(&mut self) -> Result<Stmt<'a>> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression })
    }

    fn expression(&mut self) -> Result<Expr<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr<'a>> {
        let expr = self.equality()?;

        if self.matches(&[TokenType::Equal]) {
            self.token_iter.next();
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
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

    fn equality(&mut self) -> Result<Expr<'a>> {
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

    fn comparison(&mut self) -> Result<Expr<'a>> {
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

    fn addition(&mut self) -> Result<Expr<'a>> {
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

    fn multiplication(&mut self) -> Result<Expr<'a>> {
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

    fn unary(&mut self) -> Result<Expr<'a>> {
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
                _ => self.primary(),
            }
        } else {
            unreachable!();
        }
    }

    fn primary(&mut self) -> Result<Expr<'a>> {
        if let Some(token) = self.token_iter.next() {
            match &token.token_type {
                TokenType::False => Ok(Expr::Boolean(false)),
                TokenType::True => Ok(Expr::Boolean(true)),
                TokenType::Nil => Ok(Expr::Nil),
                TokenType::Number(num) => Ok(Expr::Number(*num)),
                TokenType::String(string) => Ok(Expr::String(*string)),
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
                TokenType::Identifier => Ok(Expr::Variable { name: token.lexeme }),
                _ => Parser::expected_expression(None),
            }
        } else {
            Parser::expected_expression(None)
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

    fn expected_expression(line: Option<u32>) -> Result<Expr<'a>> {
        Err(LoxError::ParserError(
            line,
            "Unexpected end of file, expected expression.".into(),
        ))
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt<'a>>;

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

pub fn parse<'a>(tokens: &'a Vec<Token<'a>>) -> (Vec<Stmt<'a>>, Vec<LoxError>) {
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
}
