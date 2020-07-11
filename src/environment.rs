use crate::error::{LoxError, Result};
use crate::object::Object;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Object>,
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

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &str, value: Object) -> Result<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), value);
            Ok(())
        } else if let Some(enclosing) = self.enclosing.as_ref() {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(LoxError::EnvironmentError(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }

    pub fn get(&self, name: &str) -> Result<Object> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(enclosing) = self.enclosing.as_ref() {
            enclosing.borrow().get(name)
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

        first.borrow_mut().define("answer", Object::Number(42.0));

        assert_eq!(second.get("answer").unwrap(), Object::Number(42.0));
    }
}
