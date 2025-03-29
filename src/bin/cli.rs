//! A Redis CLI application.
//!
//! This application is a simple command-line interface for interacting with a Redis database.
//! It allows users to connect to a Redis server, send commands, and receive responses.
//! It is built using the `redis-async` lib crate in this repository, which provides a high-level API for working with Redis.
//! The CLI can operate in both interactive and non-interactive modes.
//! In interactive mode, users can enter commands directly into the terminal.
//! In non-interactive mode, commands can be passed as arguments.
//! The application supports various Redis commands, including:
//! - `PING`: Check if the server is alive.
//! - `GET`: Retrieve the value of a key.
//! - `SET`: Set the value of a key.
//! - `DEL`: Delete a key.
//! - `EXISTS`: Check if a key exists.
//! - `INFO`: Get information about the server.
//! - `FLUSHDB`: Flush the current database.
//! - `FLUSHALL`: Flush all databases.
//! - `KEYS`: Get all keys matching a pattern.
//! - `SCAN`: Scan the keys in the database.
//! - `HGET`: Get the value of a field in a hash.
//! - `HSET`: Set the value of a field in a hash.
//! - `HDEL`: Delete a field in a hash.
//! - `HGETALL`: Get all fields and values in a hash.
//! - `LPUSH`: Push a value onto a list.
//! - `RPUSH`: Push a value onto a list.
//! - `LPOP`: Pop a value from a list.
//! - `RPOP`: Pop a value from a list.
//! - `LRANGE`: Get a range of values from a list.
//! - `SADD`: Add a member to a set.
//! - `SREM`: Remove a member from a set.
//! - `SMEMBERS`: Get all members of a set.
//! - `ZADD`: Add a member to a sorted set.
//! - `ZREM`: Remove a member from a sorted set.
//! - `ZRANGE`: Get a range of members from a sorted set.
//! - `ZRANK`: Get the rank of a member in a sorted set.
//! - `ZREVRANK`: Get the reverse rank of a member in a sorted set.
//! - `ZCARD`: Get the number of members in a sorted set.
//! - `ZCOUNT`: Get the number of members in a sorted set with scores within a given range.
//! - `ZINCRBY`: Increment the score of a member in a sorted set.

use redis_async::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:6379").await?;

    let _ = client.ping(Some("Hello, Redis!")).await?;

    let _ = client.set("mykey", "myvalue").await?;

    let _ = client.get("mykey").await?;

    Ok(())
}
