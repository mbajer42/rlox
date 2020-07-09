use crate::environment::Environment;
use crate::error::{LoxError, Result};
use crate::object::Object;
use crate::statement::{Expr, Stmt};
use crate::token::TokenType;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::Print { expression } => {
                println!("{}", self.evaluate(&expression)?);
                Ok(())
            }
            Stmt::Expression { expression } => {
                self.evaluate(&expression)?;
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expression) = initializer {
                    self.evaluate(&expression)?
                } else {
                    Object::Nil
                };
                self.environment.define(name, value);
                Ok(())
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Nil => Ok(Object::Nil),
            Expr::Boolean(b) => Ok(Object::Boolean(*b)),
            Expr::String(s) => Ok(Object::String(s.to_string())),
            Expr::Number(num) => Ok(Object::Number(*num)),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { token_type, right } => self.unary_expression(token_type, right),
            Expr::Binary {
                left,
                token_type,
                right,
            } => self.binary_expression(left, token_type, right),
            Expr::Variable { name } => self.environment.get(name),
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

    fn cast_operands_to_numbers(&self, left: &Object, right: &Object) -> Result<(f64, f64)> {
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

    fn cast_operands_to_strings<'b>(
        &self,
        left: &'b Object,
        right: &'b Object,
    ) -> Result<(&'b String, &'b String)> {
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

#[cfg(test)]
mod tests {

    use super::Interpreter;
    use crate::lexer;
    use crate::object::Object;
    use crate::parser;
    use crate::statement::Stmt;

    #[test]
    fn simple_mathematical_expression() {
        let source = "(3 + 4) * 6;";
        let (tokens, _) = lexer::lex(source);
        let (statements, _) = parser::parse(&tokens);

        if let Stmt::Expression { expression } = &statements[0] {
            let interpreter = Interpreter::new();
            let result = interpreter.evaluate(expression).unwrap();
            assert_eq!(result, Object::Number(42.0));
        } else {
            unreachable!();
        }
    }

    #[test]
    fn assignments() {
        let source = r#"
            var halfTruth = 7 * 3;
            var answer = halfTruth * 2;
        "#;
        let (tokens, _) = lexer::lex(source);
        let (statements, _) = parser::parse(&tokens);

        let mut interpreter = Interpreter::new();
        interpreter.interpret(statements).unwrap();

        let half_truth = interpreter.environment.get("halfTruth").unwrap();
        assert_eq!(half_truth, Object::Number(21.0));

        let answer = interpreter.environment.get("answer").unwrap();
        assert_eq!(answer, Object::Number(42.0));
    }
}
