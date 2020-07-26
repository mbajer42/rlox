use crate::error::{LoxError, Result};
use crate::object::Object;
use crate::resolver::Depth;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Rc<Object>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(environment: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(environment),
        }
    }

    pub fn define(&mut self, name: &str, value: Rc<Object>) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, depth: Depth, name: &str, value: Rc<Object>) -> Result<()> {
        if depth == 0 {
            self.assign_here(name, value)
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign(depth - 1, name, value)
        }
    }

    fn assign_here(&mut self, name: &str, value: Rc<Object>) -> Result<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), value);
            Ok(())
        } else {
            Err(LoxError::EnvironmentError(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }

    pub fn get(&self, depth: Depth, name: &str) -> Result<Rc<Object>> {
        if depth == 0 {
            self.get_here(name)
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get(depth - 1, name)
        }
    }

    fn get_here(&self, name: &str) -> Result<Rc<Object>> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else {
            Err(LoxError::EnvironmentError(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Environment;
    use crate::object::Object;

    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn get() {
        let first = Rc::new(RefCell::new(Environment::new()));
        let second = Environment::with_enclosing(first.clone());

        first
            .borrow_mut()
            .define("answer", Rc::new(Object::Number(42.0)));

        assert_eq!(*second.get(1, "answer").unwrap(), Object::Number(42.0));
    }
}
