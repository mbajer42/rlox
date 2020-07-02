use crate::error::{LoxError, Result};
use crate::lexer::{Token, TokenType};

#[derive(Debug)]
pub enum Expr<'a> {
    // literal values
    Number(f64),
    String(&'a str),
    Boolean(bool),
    Nil,
    // compound expressions
    Binary(Box<Expr<'a>>, TokenType<'a>, Box<Expr<'a>>),
    Grouping(Box<Expr<'a>>),
    Unary(TokenType<'a>, Box<Expr<'a>>),
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

    fn expression(&mut self) -> Result<Expr<'a>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.comparison()?;

        while let Some(&token) = self.token_iter.peek() {
            match &token.token_type {
                TokenType::BangEqual | TokenType::LessEqual => {
                    self.token_iter.next();
                    let right = self.addition()?;
                    expr = Expr::Binary(Box::new(expr), token.token_type.clone(), Box::new(right));
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
                    expr = Expr::Binary(Box::new(expr), token.token_type.clone(), Box::new(right));
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
                    expr = Expr::Binary(Box::new(expr), token.token_type.clone(), Box::new(right));
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
                    expr = Expr::Binary(Box::new(expr), token.token_type.clone(), Box::new(right));
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
                    Ok(Expr::Unary(token.token_type.clone(), Box::new(right)))
                }
                _ => self.primary(),
            }
        } else {
            unreachable!();
        }
    }

    fn primary(&mut self) -> Result<Expr<'a>> {
        if let Some(&token) = self.token_iter.peek() {
            self.token_iter.next();
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
                            Ok(Expr::Grouping(Box::new(expr)))
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
                _ => Parser::expected_expression(None),
            }
        } else {
            Parser::expected_expression(None)
        }
    }

    fn expected_expression(line: Option<u32>) -> Result<Expr<'a>> {
        Err(LoxError::ParserError(line, "Expected expression".into()))
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Expr<'a>>;

    fn next(&mut self) -> Option<Result<Expr<'a>>> {
        match self.token_iter.peek() {
            None
            | Some(Token {
                token_type: TokenType::Eof,
                ..
            }) => None,
            _ => Some(self.expression()),
        }
    }
}

pub fn parse<'a>(tokens: &'a Vec<Token<'a>>) -> (Vec<Expr<'a>>, Vec<LoxError>) {
    let parser = Parser::new(tokens);
    let (expressions, errors): (Vec<_>, Vec<_>) = parser.partition(Result::is_ok);
    let expressions = expressions.into_iter().map(Result::unwrap).collect();
    let errors = errors.into_iter().map(Result::unwrap_err).collect();

    (expressions, errors)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lexer;

    #[test]
    fn simple_mathematical_expression() {
        let source = "(3 + 4) * 6";
        let (tokens, _) = lexer::lex(source);
        let (expressions, errors) = parse(&tokens);
        assert_eq!(errors.len(), 0);
        assert_eq!(expressions.len(), 1);
        assert_eq!(
            format!("{:?}", expressions[0]),
            "Binary(Grouping(Binary(Number(3.0), Plus, Number(4.0))), Star, Number(6.0))"
        );
    }
}
