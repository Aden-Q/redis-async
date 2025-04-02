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
//! RESP version is set per connection. By default, the connection runs in RESP2 mode. RESP3 can be
//! enabled by sending `HELLO 3` to the server. You can use `client.hello(Some(3))` to achieve it.
//! Note that RESP3 is only available in Redis 6.0 and later.

mod connection;
pub use connection::Connection;

mod frame;
pub use frame::Frame;

mod cmd;
pub use cmd::Expiry;

mod client;
pub use client::Client;

mod error;
pub use error::{RedisError, Result};
