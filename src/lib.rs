//! An asynchronous Redis client library for Rust.
//!
//! # Basic usage
//!
//! ## Example
//!
//! ```ignore
//! use async_redis::Client;
//! ```
//!
//! # TLS/SSL
//!
//! # Connection pooling
//!
//! # Asynchronous operations
//!
//! By default, the client runs in asynchronous mode. This means that all
//! operations are non-blocking and return a `Future` that can be awaited.
//!
//! # Pipelining
//!
//! # Transaction
//!
//! # Pub/Sub
//!
//! # RESP3
//!
//! This library supports the Redis Serialization Protocol (RESP) version 3
//! introduced in Redis 6.0.

mod connection;
pub use connection::Connection;

pub mod frame;
pub use frame::Frame;

mod cmd;
pub use cmd::Command;

mod client;
pub use client::Client;

mod error;
pub use error::{RedisError, Result};
