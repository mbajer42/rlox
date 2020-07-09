use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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
        }
    }
}
