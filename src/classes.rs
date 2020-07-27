use crate::error::{LoxError, Result};
use crate::functions::LoxFunction;
use crate::object::Object;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct LoxClass {
    name: String,
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        if let Some(method) = self.methods.get(name) {
            Some(Rc::clone(method))
        } else if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
        }
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

    pub fn get(wrapping_object: Rc<Object>, name: &str) -> Result<Rc<Object>> {
        if let Object::Instance(instance) = wrapping_object.clone().as_ref() {
            if let Some(value) = instance.borrow().fields.get(name) {
                Ok(Rc::clone(value))
            } else if let Some(method) = instance.borrow().class.as_ref().find_method(name) {
                Ok(Rc::new(Object::Function(Rc::new(
                    method.bind(wrapping_object),
                ))))
            } else {
                Err(LoxError::InterpreterError(
                    format!("Undefined property {}.", name).into(),
                ))
            }
        } else {
            Err(LoxError::InterpreterError(
                "Only instances have fields.".into(),
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
