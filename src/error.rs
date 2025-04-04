//! Custom error handling for Redis client and a specialized Result type
//! used as the return type for Redis operations.

/// Represents errors that can occur when working with Redis.
#[derive(thiserror::Error, Debug)]
pub enum RedisError {
    #[error("error from io")]
    Io(#[from] std::io::Error),
    /// An incomplete frame was received when reading from the socket.
    #[error("incomplete frame")]
    IncompleteFrame,
    /// An invalid frame was received when reading from the socket. According to RESP3 spec.
    #[error("invalid frame")]
    InvalidFrame,
    /// So that we can use `?` operator to convert from `std::str::Utf8Error`
    #[error("utf8 error")]
    Utf8(#[from] std::str::Utf8Error),
    /// So that we can use `?` operator to convert from `std::num::ParseIntError`
    #[error("ParseIntError")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("TryFromIntError")]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[error("unexpected response type")]
    UnexpectedResponseType,
    /// All other errors are converted to anyhow::Error
    /// This is a catch-all error type that can be used to wrap any other error.
    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
    /// Last resort error type. This is used when we don't know what went wrong.
    /// Should avoid using this error type if possible.
    #[error("unknown error")]
    Unknown,
}

/// A specialized `Result` type for Redis operations.
pub type Result<T> = anyhow::Result<T, RedisError>;
