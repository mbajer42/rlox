use crate::error::{LoxError, Result};
use crate::object::Object;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Rc<Object>>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Result<Rc<Object>> {
        if let Some(value) = self.fields.get(name) {
            Ok(Rc::clone(value))
        } else {
            Err(LoxError::InterpreterError(
                format!("Undefined property {}.", name).into(),
            ))
        }
    }

    pub fn set(&mut self, name: &str, value: Rc<Object>) {
        self.fields.insert(name.to_string(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
