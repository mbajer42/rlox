use crate::environment::Environment;
use crate::error::{LoxError, Result};
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::statement::Stmt;

use std::cell::RefCell;
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
}

impl LoxFunction {
    pub fn new(
        parameters: Rc<Vec<String>>,
        body: Rc<Vec<Stmt>>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        LoxFunction {
            parameters,
            body,
            closure,
        }
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
        let mut environment = Environment::with_enclosing(self.closure.clone());
        self.parameters
            .iter()
            .zip(arguments.iter())
            .for_each(|(declaration, argument)| {
                environment.define(declaration, argument.clone());
            });

        let result = interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment)));
        let return_value = match result {
            Ok(()) => Rc::new(Object::Nil),
            Err(LoxError::Return(value)) => value,
            Err(err) => return Err(err),
        };

        Ok(return_value)
    }
}
