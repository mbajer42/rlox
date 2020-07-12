use crate::error::Result;
use crate::interpreter::Interpreter;
use crate::object::Object;

use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Rc<Object>>) -> Result<Object>;
}

impl std::fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lox fn>")
    }
}

pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &mut Interpreter, _: &Vec<Rc<Object>>) -> Result<Object> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Ok(Object::Number(now.as_secs() as f64))
    }
}

impl std::fmt::Debug for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
