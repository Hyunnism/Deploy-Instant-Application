use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum AppError {
    Internal(String),
    NotFound(String),
    Validation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<T> = std::result::Result<T, AppError>;
