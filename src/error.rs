//! Custom error handling for Redis client and a specialized Result type
//! used as the return type for Redis operations.
//!
//! todo: implement From trait for RedisError so that we can capture more built in e

use std::{error, fmt, io, result};

/// Represents errors that can occur when working with Redis.
#[derive(Debug)]
pub enum RedisError {
    /// An incomplete frame was received when reading from the socket.
    IncompleteFrame,
    /// An invalid frame was received when reading from the socket. According to RESP3 spec.
    InvalidFrame,
    /// Generic error type.
    Other(Error),
}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RedisError::IncompleteFrame => write!(f, "incomplete frame"),
            RedisError::InvalidFrame => write!(f, "invalid frame"),
            RedisError::Other(s) => write!(f, "{:?}", s),
        }
    }
}

// Implement std::error::Error for RedisError.
impl error::Error for RedisError {}

impl From<io::Error> for RedisError {
    fn from(err: io::Error) -> Self {
        RedisError::Other(err.into())
    }
}

impl From<String> for RedisError {
    fn from(val: String) -> Self {
        RedisError::Other(val.into())
    }
}

impl From<&str> for RedisError {
    fn from(val: &str) -> Self {
        RedisError::Other(val.into())
    }
}

/// Boxed generic error types.
type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for Redis operations.
pub type Result<T> = result::Result<T, Error>;

/// Helper function to wrap errors into Box.
pub fn wrap_error<E: Into<RedisError>>(err: E) -> Error {
    Box::new(err.into())
}
