use crate::error::{LoxError, Result};
use crate::object::Object;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &str) -> Result<Object> {
        match self.values.get(name) {
            Some(value) => Ok(value.clone()),
            None => Err(LoxError::EnvironmentError(format!(
                "Undefined variable '{}'.",
                name
            ))),
        }
    }
}
