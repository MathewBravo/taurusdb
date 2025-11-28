use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum StorageError {
    DecodeError(String),
}

impl Error for StorageError {}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DecodeError(err) => write!(f, "Decode Error: {}", err),
        }
    }
}
