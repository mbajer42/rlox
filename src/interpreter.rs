use crate::environment::Environment;
use crate::error::{LoxError, Result};
use crate::object::Object;
use crate::statement::{Expr, Stmt};
use crate::token::TokenType;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in &statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
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
                self.environment.borrow_mut().define(name, value);
                Ok(())
            }
            Stmt::Block { statements } => self.execute_block(&*statements),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;
                if self.is_truthy(&condition) {
                    self.execute(&*then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(&*else_branch)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(Environment::with_enclosing(previous.clone())));

        for statement in statements {
            self.execute(statement)?;
        }

        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
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
            Expr::Variable { name } => self.environment.borrow().get(name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                if operator == &TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }
                self.evaluate(right)
            }
        }
    }

    fn unary_expression(&mut self, token_type: &TokenType, expr: &Expr) -> Result<Object> {
        let right = self.evaluate(expr)?;

        match token_type {
            TokenType::Minus => match right {
                Object::Number(num) => Ok(Object::Number(-num)),
                _ => Err(LoxError::InterpreterError(
                    format!("Operand must be a number, but got '{}'", right).into(),
                )),
            },
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&right))),
            _ => unreachable!(),
        }
    }

    fn binary_expression(
        &mut self,
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

    fn is_truthy(&self, object: &Object) -> bool {
        match *object {
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
            let mut interpreter = Interpreter::new();
            let result = interpreter.evaluate(expression).unwrap();
            assert_eq!(result, Object::Number(42.0));
        } else {
            unreachable!();
        }
    }

    #[test]
    fn var_declaration() {
        let source = r#"
            var half = 7 * 3;
            var answer = half * 2;
        "#;
        let (tokens, _) = lexer::lex(source);
        let (statements, _) = parser::parse(&tokens);

        let mut interpreter = Interpreter::new();
        interpreter.interpret(statements).unwrap();

        let half_truth = interpreter.environment.borrow().get("half").unwrap();
        assert_eq!(half_truth, Object::Number(21.0));

        let answer = interpreter.environment.borrow().get("answer").unwrap();
        assert_eq!(answer, Object::Number(42.0));
    }

    #[test]
    fn block() {
        let source = r#"
            var answer = 42;
            var thirteen = 0;
            {
                var answer = 21;
                thirteen = 13;
                var lost = "lost";
            }
        "#;

        let (tokens, _) = lexer::lex(source);
        let (statements, _) = parser::parse(&tokens);

        let mut interpreter = Interpreter::new();
        interpreter.interpret(statements).unwrap();

        let answer = interpreter.environment.borrow().get("answer").unwrap();
        assert_eq!(answer, Object::Number(42.0));

        let thirteen = interpreter.environment.borrow().get("thirteen").unwrap();
        assert_eq!(thirteen, Object::Number(13.0));

        assert!(interpreter.environment.borrow().get("lost").is_err());
    }

    #[test]
    fn if_statement() {
        let source = r#"
            var truth = true;
            var answer;
            if (truth) {
                answer = 42;
            } else {
                answer = 21;
            }
        "#;

        let (tokens, _) = lexer::lex(source);
        let (statements, parser_errs) = parser::parse(&tokens);
        for err in parser_errs {
            println!("{}", err);
        }

        let mut interpreter = Interpreter::new();
        interpreter.interpret(statements).unwrap();

        let answer = interpreter.environment.borrow().get("answer").unwrap();
        assert_eq!(answer, Object::Number(42.0));
    }
}
