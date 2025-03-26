use std::{error, fmt, io, result, sync};

// Custom error enum
#[derive(Debug)]
pub enum RedisError {
    Io(io::Error),
    IncompleteFrame,
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

// Implement std::error::Error for RedisError
impl error::Error for RedisError {}

type Error = sync::Arc<RedisError>;

// Helper function to wrap errors into Arc
pub fn wrap_error<E: Into<RedisError>>(err: E) -> Error {
    sync::Arc::new(err.into())
}

/// A specialized `Result` type for Redis operations.
///
/// This is defined as a convenience.
pub type Result<T> = result::Result<T, Error>;
