pub mod function;
pub use function::Function;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MiniModalError {
    CompileError(String),
    FunctionError(String),
    ServerError(String),
    OtherError(String),
    ConnectionError(String),
    SerializationError(String),
}

impl Display for MiniModalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<anyhow::Error> for MiniModalError {
    fn from(error: anyhow::Error) -> Self {
        MiniModalError::OtherError(error.to_string())
    }
}

impl From<std::io::Error> for MiniModalError {
    fn from(error: std::io::Error) -> Self {
        MiniModalError::OtherError(error.to_string())
    }
}

use tonic::transport::Error as TonicError;
use serde_json::Error as SerdeJsonError;

// Add these implementations somewhere in your code, possibly in the same file or in a separate error module
impl From<TonicError> for MiniModalError {
    fn from(error: TonicError) -> Self {
        MiniModalError::ConnectionError(error.to_string())
    }
}

impl From<SerdeJsonError> for MiniModalError {
    fn from(error: SerdeJsonError) -> Self {
        MiniModalError::SerializationError(error.to_string())
    }
}