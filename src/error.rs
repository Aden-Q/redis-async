//! Custom error handling for Redis client and a specialized Result type
//! used as the return type for Redis operations.

use std::{error, fmt, io, result, sync};

/// Represents errors that can occur when working with Redis.
#[derive(Debug)]
pub enum RedisError {
    /// An I/O error that occurred while working with a Redis connection.
    Io(io::Error),
    /// An incomplete frame was received when reading from the socket.
    IncompleteFrame,
    /// An invalid frame was received when reading from the socket. According to RESP3 spec.
    InvalidFrame,
    Other(String),
}

impl From<io::Error> for RedisError {
    fn from(err: io::Error) -> Self {
        RedisError::Io(err)
    }
}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RedisError::Io(err) => write!(f, "IO error: {}", err),
            RedisError::IncompleteFrame => write!(f, "incomplete frame"),
            RedisError::InvalidFrame => write!(f, "invalid frame"),
            RedisError::Other(s) => write!(f, "other error: {}", s),
        }
    }
}

// Implement std::error::Error for RedisError.
impl error::Error for RedisError {}

type Error = sync::Arc<RedisError>;

/// Helper function to wrap errors into Arc.
pub fn wrap_error<E: Into<RedisError>>(err: E) -> Error {
    sync::Arc::new(err.into())
}

/// A specialized `Result` type for Redis operations.
pub type Result<T> = result::Result<T, Error>;
