use crate::callable::Callable;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Callable(Box<dyn Callable>),
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
            Object::Callable(func) => write!(f, "{:?}", func),
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
            (Object::Callable(_), Object::Callable(_)) => false,
            _ => false,
        }
    }
}
