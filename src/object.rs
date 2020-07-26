use crate::classes::{LoxClass, LoxInstance};
use crate::functions::Function;

use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Function(Rc<dyn Function>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Object::Nil => write!(f, "nil"),
            Object::Number(num) => {
                if num.fract() == 0.0 {
                    write!(f, "{:.0}", num)
                } else {
                    write!(f, "{}", num)
                }
            }
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
            Object::Function(func) => write!(f, "{:?}", func),
            Object::Class(class) => write!(f, "{}", class),
            Object::Instance(instance) => write!(f, "{}", instance.borrow()),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::Nil, Object::Nil) => true,
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            _ => false,
        }
    }
}
