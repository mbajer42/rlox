use crate::classes::{LoxClass, LoxInstance};
use crate::environment::Environment;
use crate::error::{LoxError, Result};
use crate::functions::{Clock, Function, LoxFunction};
use crate::object::Object;
use crate::resolver::Depth;
use crate::statement::{Expr, ExprId, Stmt};
use crate::token::TokenType;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    scopes: HashMap<ExprId, Depth>,
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        globals
            .borrow_mut()
            .define("clock", Rc::new(Object::Function(Rc::new(Clock {}))));

        Interpreter {
            scopes: HashMap::new(),
            globals: globals.clone(),
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(&statement)?;
        }
        Ok(())
    }

    pub fn add_scopes(&mut self, scopes: HashMap<ExprId, Depth>) {
        scopes.iter().for_each(|(&k, &v)| {
            self.scopes.insert(k, v);
        });
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
                    Rc::new(Object::Nil)
                };
                self.environment.borrow_mut().define(&name, value);
                Ok(())
            }
            Stmt::Block { statements } => self.execute_block(
                statements,
                Rc::new(RefCell::new(Environment::with_enclosing(
                    self.environment.clone(),
                ))),
            ),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;
                if self.is_truthy(&condition) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                let mut evaluated_condition = self.evaluate(&condition)?;
                while self.is_truthy(&evaluated_condition) {
                    self.execute(body)?;
                    evaluated_condition = self.evaluate(&condition)?;
                }
                Ok(())
            }
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = Rc::new(Object::Function(Rc::new(LoxFunction::new(
                    parameters.clone(),
                    body.clone(),
                    self.environment.clone(),
                    false,
                ))));
                self.environment.borrow_mut().define(&name, function);
                Ok(())
            }
            Stmt::Return { value } => {
                let value = if let Some(value) = value {
                    self.evaluate(value)?
                } else {
                    Rc::new(Object::Nil)
                };
                Err(LoxError::Return(value))
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let (super_object, super_loxclass) = if let Some(superclass) = superclass {
                    let super_object = self.evaluate(superclass)?;
                    if let Object::Class(super_loxclass) = super_object.as_ref() {
                        (Some(Rc::clone(&super_object)), Some(super_loxclass.clone()))
                    } else {
                        return Err(LoxError::InterpreterError(
                            "Superclass must be a class".into(),
                        ));
                    }
                } else {
                    (None, None)
                };

                let method_environment = if super_object.is_some() {
                    let env = Rc::new(RefCell::new(Environment::with_enclosing(Rc::clone(
                        &self.environment,
                    ))));
                    env.borrow_mut()
                        .define("super", Rc::clone(&super_object.unwrap()));
                    env
                } else {
                    Rc::clone(&self.environment)
                };

                let mut name_to_method = HashMap::new();
                for method in methods.as_ref() {
                    if let Stmt::Function {
                        name,
                        parameters,
                        body,
                    } = method
                    {
                        name_to_method.insert(
                            name.to_string(),
                            Rc::new(LoxFunction::new(
                                parameters.clone(),
                                body.clone(),
                                Rc::clone(&method_environment),
                                name == "init",
                            )),
                        );
                    } else {
                        unreachable!()
                    }
                }

                let class = Rc::new(Object::Class(Rc::new(LoxClass::new(
                    name.to_string(),
                    super_loxclass,
                    name_to_method,
                ))));

                self.environment.borrow_mut().define(name, class);

                Ok(())
            }
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = environment;

        for statement in statements {
            self.execute(statement).map_err(|err| {
                self.environment = previous.clone();
                err
            })?;
        }

        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Rc<Object>> {
        match expr {
            Expr::Nil => Ok(Rc::new(Object::Nil)),
            Expr::Boolean(b) => Ok(Rc::new(Object::Boolean(*b))),
            Expr::String(s) => Ok(Rc::new(Object::String(s.to_string()))),
            Expr::Number(num) => Ok(Rc::new(Object::Number(*num))),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { token_type, right } => self.unary_expression(token_type, right),
            Expr::Binary {
                left,
                token_type,
                right,
            } => self.binary_expression(left, token_type, right),
            Expr::Variable { id, name } => {
                let depth = self.get_locals_depth(id);
                if let Some(depth) = depth {
                    self.environment.borrow().get(depth, name)
                } else {
                    self.globals.borrow().get(0, name)
                }
            }
            Expr::This { id, keyword } => {
                let depth = self.get_locals_depth(id);
                if let Some(depth) = depth {
                    self.environment.borrow().get(depth, keyword)
                } else {
                    self.globals.borrow().get(0, keyword)
                }
            }
            Expr::Super {
                id,
                keyword,
                method: method_name,
            } => {
                let depth = self.get_locals_depth(id).unwrap();
                let superclass = self.environment.borrow().get(depth, keyword)?;

                // "this" is always one depth closer than "super"'s environment
                let superobject = self.environment.borrow().get(depth - 1, "this")?;
                if let Object::Class(superclass) = superclass.as_ref() {
                    let method = superclass.find_method(method_name);
                    if let Some(method) = method {
                        Ok(Rc::new(Object::Function(Rc::new(method.bind(superobject)))))
                    } else {
                        Err(LoxError::InterpreterError(
                            format!("Undefined property '{}'.", method_name).into(),
                        ))
                    }
                } else {
                    unreachable!()
                }
            }
            Expr::Assign { id, name, value } => {
                let value = self.evaluate(value)?;
                let depth = self.get_locals_depth(id);
                if let Some(depth) = depth {
                    self.environment
                        .borrow_mut()
                        .assign(depth, name, value.clone())?;
                } else {
                    self.globals.borrow_mut().assign(0, name, value.clone())?;
                }
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
            Expr::Call { callee, arguments } => self.call_expression(callee, arguments),
            Expr::Get { object, name } => {
                let object = self.evaluate(object)?;
                LoxInstance::get(object, name)
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let object = self.evaluate(object)?;
                let value = self.evaluate(value)?;
                if let &Object::Instance(ref instance) = object.as_ref() {
                    instance.borrow_mut().set(name, value);
                    Ok(Rc::new(Object::Nil))
                } else {
                    Err(LoxError::InterpreterError(
                        "Only instances have fields.".into(),
                    ))
                }
            }
        }
    }

    fn unary_expression(&mut self, token_type: &TokenType, expr: &Expr) -> Result<Rc<Object>> {
        let right = self.evaluate(expr)?;

        match token_type {
            TokenType::Minus => match *right {
                Object::Number(num) => Ok(Rc::new(Object::Number(-num))),
                _ => Err(LoxError::InterpreterError(
                    format!("Operand must be a number, but got '{}'", right).into(),
                )),
            },
            TokenType::Bang => Ok(Rc::new(Object::Boolean(!self.is_truthy(&right)))),
            _ => unreachable!(),
        }
    }

    fn binary_expression(
        &mut self,
        left: &Expr,
        token_type: &TokenType,
        right: &Expr,
    ) -> Result<Rc<Object>> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match token_type {
            TokenType::Star => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Number(left * right)))
            }
            TokenType::Minus => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Number(left - right)))
            }
            TokenType::Slash => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Number(left / right)))
            }
            TokenType::Plus => {
                if let Ok((left, right)) = self.cast_operands_to_numbers(&left, &right) {
                    Ok(Rc::new(Object::Number(left + right)))
                } else if let Ok((left, right)) = self.cast_operands_to_strings(&left, &right) {
                    Ok(Rc::new(Object::String(format!("{}{}", left, right))))
                } else {
                    Err(LoxError::InterpreterError(format!(
                        "The '+' operator requires either 2 numbers or 2 strings, but got '{}' and '{}'",
                        &left, &right
                    ).into()))
                }
            }
            TokenType::LessEqual => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Boolean(left <= right)))
            }
            TokenType::Less => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Boolean(left < right)))
            }
            TokenType::GreaterEqual => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Boolean(left >= right)))
            }
            TokenType::Greater => {
                let (left, right) = self.cast_operands_to_numbers(&left, &right)?;
                Ok(Rc::new(Object::Boolean(left > right)))
            }
            _ => unreachable!(),
        }
    }

    fn call_expression(&mut self, callee: &Expr, arguments: &Vec<Expr>) -> Result<Rc<Object>> {
        let callee = self.evaluate(callee)?;

        let arguments = arguments
            .iter()
            .map(|argument| self.evaluate(argument))
            .collect::<Result<Vec<_>>>()?;

        match callee.as_ref() {
            Object::Function(function) => Ok(function.call(self, &arguments)?),
            Object::Class(class) => {
                let instance = Rc::new(Object::Instance(Rc::new(RefCell::new(LoxInstance::new(
                    Rc::clone(class),
                )))));
                let constructor = class.find_method("init");
                if let Some(constructor) = constructor {
                    constructor
                        .bind(Rc::clone(&instance))
                        .call(self, &arguments)?;
                }
                Ok(instance)
            }
            _ => Err(LoxError::InterpreterError(
                "Can only call functions and classes.".into(),
            )),
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

    fn get_locals_depth(&self, expression_id: &ExprId) -> Option<u64> {
        self.scopes.get(expression_id).copied()
    }
}

#[cfg(test)]
mod tests {

    use super::Interpreter;
    use crate::lexer;
    use crate::object::Object;
    use crate::parser;
    use crate::resolver;
    use crate::statement::Stmt;

    fn interpret(source: &'static str) -> Interpreter {
        let (tokens, lexer_errors) = lexer::lex(source);
        assert_eq!(lexer_errors.len(), 0);
        let (statements, parser_errors) = parser::parse(&tokens);
        assert_eq!(parser_errors.len(), 0);

        let scopes = resolver::resolve(&statements);
        assert_eq!(scopes.is_ok(), true);

        let mut interpreter = Interpreter::new();
        interpreter.add_scopes(scopes.unwrap());
        interpreter.interpret(statements).unwrap();

        interpreter
    }

    #[test]
    fn simple_mathematical_expression() {
        let source = "(3 + 4) * 6;";
        let (tokens, _) = lexer::lex(source);
        let (statements, _) = parser::parse(&tokens);

        if let Stmt::Expression { expression } = &statements[0] {
            let mut interpreter = Interpreter::new();
            let result = interpreter.evaluate(expression).unwrap();
            assert_eq!(*result, Object::Number(42.0));
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
        let interpreter = interpret(source);

        let half_truth = interpreter.environment.borrow().get(0, "half").unwrap();
        assert_eq!(*half_truth, Object::Number(21.0));

        let answer = interpreter.environment.borrow().get(0, "answer").unwrap();
        assert_eq!(*answer, Object::Number(42.0));
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
        let interpreter = interpret(source);

        let answer = interpreter.environment.borrow().get(0, "answer").unwrap();
        assert_eq!(*answer, Object::Number(42.0));

        let thirteen = interpreter.environment.borrow().get(0, "thirteen").unwrap();
        assert_eq!(*thirteen, Object::Number(13.0));

        assert!(interpreter.environment.borrow().get(0, "lost").is_err());
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
        let interpreter = interpret(source);

        let answer = interpreter.environment.borrow().get(0, "answer").unwrap();
        assert_eq!(*answer, Object::Number(42.0));
    }

    #[test]
    fn while_statement() {
        let source = r#"
            var prev = 0;
            var current = 1;

            var i = 2;
            while (i < 10) {
                var temp = current;
                current = current + prev;
                prev = temp;
                i = i + 1;
            }
        "#;
        let interpreter = interpret(source);

        let current_fib = interpreter.environment.borrow().get(0, "current").unwrap();
        assert_eq!(*current_fib, Object::Number(34.0));
    }

    #[test]
    fn for_statement() {
        let source = r#"
            var product = 1;
            for (var i = 1; i <= 10; i = i + 1) {
                product = product * i;
            }
        "#;
        let interpreter = interpret(source);

        let product = interpreter.environment.borrow().get(0, "product").unwrap();
        assert_eq!(*product, Object::Number(3628800.0));
    }

    #[test]
    fn call_clock() {
        let source = r#"
            var time = clock();
        "#;
        let interpreter = interpret(source);

        let time = interpreter.environment.borrow().get(0, "time").unwrap();
        if let &Object::Number(time) = &*time {
            assert!(time > 0.0);
        } else {
            panic!("Expected that clock() returns a number");
        }
    }

    #[test]
    fn functions() {
        let source = r#"
            fun fib(n) {
                if (n <= 1) {
                    return n;
                }
                return fib(n - 1) + fib(n - 2);
            }
            var fifth = fib(5);
        "#;
        let interpreter = interpret(source);
        let fifth_fib = interpreter.environment.borrow().get(0, "fifth").unwrap();
        assert_eq!(*fifth_fib, Object::Number(5.0));
    }

    #[test]
    fn closure() {
        let source = r#"
            fun makeCounter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    return i;
                }
                return count;
            }
            var counter = makeCounter();
            var one = counter();
            var two = counter();
        "#;
        let interpreter = interpret(source);
        let one = interpreter.environment.borrow().get(0, "one").unwrap();
        let two = interpreter.environment.borrow().get(0, "two").unwrap();
        assert_eq!(*one, Object::Number(1.0));
        assert_eq!(*two, Object::Number(2.0));
    }

    #[test]
    fn resolves() {
        let source = r#"
            var a = 1;
            {
                fun increment() {
                    a = a + 1;
                }
                var a = 1;
                increment();
            }
        "#;
        let interpreter = interpret(source);
        let a = interpreter.environment.borrow().get(0, "a").unwrap();
        assert_eq!(*a, Object::Number(2.0));
    }

    #[test]
    fn get_set_fields() {
        let source = r#"
            class Foo {}
            var foo = Foo();
            foo.field = "some value";
            var field = foo.field;
        "#;
        let interpreter = interpret(source);
        let field = interpreter.environment.borrow().get(0, "field").unwrap();
        assert_eq!(*field, Object::String("some value".to_owned()));
    }

    #[test]
    fn method_calls() {
        let source = r#"
            class Person {
                init(name) {
                    this.name = name;
                }

                hi() {
                    return "Hi, my name is " + this.name;
                }
            }
            var alice = Person("Alice");
            var hiAlice = alice.hi();
        "#;
        let interpreter = interpret(source);
        let hi = interpreter.environment.borrow().get(0, "hiAlice").unwrap();
        assert_eq!(*hi, Object::String("Hi, my name is Alice".to_owned()));
    }

    #[test]
    fn super_method_calls() {
        let source = r#"
            class Duck {
                type() {
                    return "Duck";
                }
            }

            class MallardDuck < Duck {
                type() {
                    return "Mallard" + super.type();
                }
            }

            var duck = MallardDuck();
            var type = duck.type();
        "#;
        let interpreter = interpret(source);
        let duck_type = interpreter.environment.borrow().get(0, "type").unwrap();
        assert_eq!(*duck_type, Object::String("MallardDuck".to_owned()));
    }
}
