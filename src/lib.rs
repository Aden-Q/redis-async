//! An asynchronous Redis client library for Rust.
//!
//! # Basic usage
//!
//! ## Examples
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
//! # RESP2/RESP3
//!
//! RESP version is set per connection. The clients default to RESP2, RESP3 can be
//! enabled by sending `HELLO 3` to the server. Note that RESP3 is only available in
//! Redis 6.0 and later.
//!
//! This library supports the Redis Serialization Protocol (RESP) version 3
//! introduced in Redis 6.0.

mod connection;
pub use connection::Connection;

mod frame;
pub use frame::Frame;

mod cmd;

mod client;
pub use client::Client;

mod error;
pub use error::{RedisError, Result};
