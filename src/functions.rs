use crate::environment::Environment;
use crate::error::{LoxError, Result};
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::statement::Stmt;

use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Function {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Rc<Object>>,
    ) -> Result<Rc<Object>>;
}

impl std::fmt::Debug for dyn Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lox fn>")
    }
}

pub struct Clock;

impl Function for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &mut Interpreter, _: &Vec<Rc<Object>>) -> Result<Rc<Object>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Ok(Rc::new(Object::Number(now.as_secs() as f64)))
    }
}

impl std::fmt::Debug for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

pub struct LoxFunction {
    parameters: Rc<Vec<String>>,
    body: Rc<Vec<Stmt>>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        parameters: Rc<Vec<String>>,
        body: Rc<Vec<Stmt>>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        LoxFunction {
            parameters,
            body,
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Rc<Object>) -> Self {
        let mut environment = Environment::with_enclosing(self.closure.clone());
        environment.define("this", instance);
        Self {
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            closure: Rc::new(RefCell::new(environment)),
            is_initializer: self.is_initializer,
        }
    }
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "LoxFunction {{ parameters: {:?}, body: {:?} }}",
            self.parameters, self.body
        )
    }
}

impl Function for LoxFunction {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Rc<Object>>,
    ) -> Result<Rc<Object>> {
        if self.arity() != arguments.len() {
            return Err(LoxError::InterpreterError(
                format!(
                    "Expected {} arguments but got {}.",
                    self.arity(),
                    arguments.len()
                )
                .into(),
            ));
        };
        let mut environment = Environment::with_enclosing(self.closure.clone());
        self.parameters
            .iter()
            .zip(arguments.iter())
            .for_each(|(declaration, argument)| {
                environment.define(declaration, argument.clone());
            });

        let result = interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment)));
        let return_value = match result {
            Ok(()) => {
                if self.is_initializer {
                    self.closure.borrow().get(0, "this")?
                } else {
                    Rc::new(Object::Nil)
                }
            }
            Err(LoxError::Return(value)) => {
                if self.is_initializer {
                    self.closure.borrow().get(0, "this")?
                } else {
                    value
                }
            }
            Err(err) => return Err(err),
        };

        Ok(return_value)
    }
}
