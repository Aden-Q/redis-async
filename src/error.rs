//! Custom error handling for Redis client and a specialized Result type
//! used as the return type for Redis operations.
//!
//! todo: implement From trait for RedisError so that we can capture more built in e

use thiserror::Error;

/// Represents errors that can occur when working with Redis.
#[derive(Error, Debug)]
pub enum RedisError {
    #[error("error from io")]
    IO(#[from] std::io::Error),
    /// An incomplete frame was received when reading from the socket.
    #[error("incomplete frame")]
    IncompleteFrame,
    /// An invalid frame was received when reading from the socket. According to RESP3 spec.
    #[error("invalid frame")]
    InvalidFrame,
    #[error("unknown error")]
    Unknown,
    #[error("utf8 error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("parseint error")]
    ParseInt(#[from] std::num::ParseIntError),
    // other error convert from string
    #[error("other error")]
    Other(String),
}

/// A specialized `Result` type for Redis operations.
pub type Result<T> = anyhow::Result<T, RedisError>;
