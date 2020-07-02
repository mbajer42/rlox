use crate::error::{LoxError, Result};
use crate::lexer::TokenType;
use crate::parser::Expr;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::Number(num) => {
                if num.fract() == 0.0 {
                    write!(f, "{:.0}", num)
                } else {
                    write!(f, "{}", num)
                }
            }
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
        }
    }
}

struct Interpreter {}

impl Interpreter {
    fn new() -> Self {
        Interpreter {}
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Nil => Ok(Object::Nil),
            Expr::Boolean(b) => Ok(Object::Boolean(*b)),
            Expr::String(s) => Ok(Object::String(s.to_string())),
            Expr::Number(num) => Ok(Object::Number(*num)),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary(token_type, expr) => self.unary_expression(token_type, expr),
            Expr::Binary(left, token_type, right) => {
                self.binary_expression(left, token_type, right)
            }
        }
    }

    fn unary_expression(&self, token_type: &TokenType, expr: &Expr) -> Result<Object> {
        let right = self.evaluate(expr)?;

        match token_type {
            TokenType::Minus => match right {
                Object::Number(num) => Ok(Object::Number(-num)),
                _ => Err(LoxError::InterpreterError(
                    format!("Operand must be a number, but got '{}'", right).into(),
                )),
            },
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(right))),
            _ => unreachable!(),
        }
    }

    fn binary_expression(
        &self,
        left: &Expr,
        token_type: &TokenType,
        right: &Expr,
    ) -> Result<Object> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match token_type {
            TokenType::Star => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Number(left * right))
            }
            TokenType::Minus => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Number(left - right))
            }
            TokenType::Slash => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Number(left / right))
            }
            TokenType::Plus => {
                if let Ok((left, right)) = self.cast_operands_to_numbers(&left, &right) {
                    Ok(Object::Number(left + right))
                } else if let Ok((left, right)) = self.cast_operands_to_strings(&left, &right) {
                    Ok(Object::String(format!("{}{}", left, right)))
                } else {
                    Err(LoxError::InterpreterError(format!(
                        "The '+' operator requires either 2 numbers or 2 strings, but got '{}' and '{}'",
                        &left, &right
                    ).into()))
                }
            }
            TokenType::LessEqual => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Boolean(left <= right))
            }
            TokenType::Less => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Boolean(left < right))
            }
            TokenType::GreaterEqual => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Boolean(left >= right))
            }
            TokenType::Greater => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Object::Boolean(left > right))
            }
            _ => unreachable!(),
        }
    }

    fn cast_operands_to_numbers<'a>(&self, left: &Object, right: &Object) -> Result<(f64, f64)> {
        match (left, right) {
            (Object::Number(a), Object::Number(b)) => Ok((*a, *b)),
            _ => Err(LoxError::InterpreterError(
                format!(
                    "Expected both operands to be numbers, but got '{}' and '{}'",
                    left, right,
                )
                .into(),
            )),
        }
    }

    fn cast_operands_to_strings<'a>(
        &self,
        left: &'a Object,
        right: &'a Object,
    ) -> Result<(&'a String, &'a String)> {
        match (left, right) {
            (Object::String(a), Object::String(b)) => Ok((a, b)),
            _ => Err(LoxError::InterpreterError(
                format!(
                    "Expected both operands to be strings, but got '{}' and '{}'",
                    left, right
                )
                .into(),
            )),
        }
    }

    fn is_truthy(&self, object: Object) -> bool {
        match object {
            Object::Nil => false,
            Object::Boolean(b) => b,
            _ => true,
        }
    }
}

pub fn interpret<'a>(expressions: &Vec<Expr<'a>>) {
    let interpreter = Interpreter::new();
    for expression in expressions {
        let result = interpreter.evaluate(expression);
        if result.is_ok() {
            println!("{}", result.unwrap())
        } else {
            println!("{}", result.unwrap_err())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{Interpreter, Object};
    use crate::{lexer, parser};

    #[test]
    fn simple_mathematical_expression() {
        let source = "(3 + 4) * (6 * 1) > 21";
        let (tokens, _) = lexer::lex(source);
        let interpreter = Interpreter::new();
        let (expressions, _) = parser::parse(&tokens);
        let object = interpreter
            .evaluate(&expressions[0])
            .expect("Should interpret without any problems");
        assert_eq!(object, Object::Boolean(true));
    }
}
