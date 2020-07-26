use crate::object::Object;

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum LoxError {
    ParserError(Option<u32>, Cow<'static, str>),
    LexerError(u32, Cow<'static, str>),
    InterpreterError(Cow<'static, str>),
    EnvironmentError(String),
    ResolverError(&'static str),
    Return(Rc<Object>),
}

impl Display for LoxError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LoxError::ParserError(Some(line), ref reason) => {
                write!(f, "Parser error in line {}: {}", line, reason)
            }
            LoxError::ParserError(None, ref reason) => {
                write!(f, "Parser error in last line: {}", reason)
            }
            LoxError::LexerError(line, ref reason) => {
                write!(f, "Lexer error in line {}: {}", line, reason)
            }
            LoxError::InterpreterError(ref reason) => write!(f, "{}", reason),
            LoxError::EnvironmentError(ref reason) => write!(f, "{}", reason),
            LoxError::ResolverError(ref reason) => write!(f, "{}", reason),
            LoxError::Return(_value) => write!(
                f,
                "Forgot to handle return statement, this should not happen"
            ),
        }
    }
}

impl std::error::Error for LoxError {}

pub type Result<T> = std::result::Result<T, LoxError>;
