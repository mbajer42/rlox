use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LoxError {
    ParserError(Option<u32>, String),
    LexerError(u32, String),
}

impl Display for LoxError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LoxError::ParserError(Some(line), ref reason) => {
                write!(f, "Error while parsing in line {}: {}", line, reason)
            }
            LoxError::ParserError(None, ref reason) => {
                write!(f, "Error while parsing in last line: {}", reason)
            }
            LoxError::LexerError(line, ref reason) => {
                write!(f, "Error while scanning in line {}: {}", line, reason)
            }
        }
    }
}

impl std::error::Error for LoxError {}

pub type Result<T> = std::result::Result<T, LoxError>;
