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

use clap::{Parser, Subcommand};
use redis_async::{Client, Result};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "redis-async-cli")]
#[command(version = "1.0.0")]
#[command(about = "redis-cli 1.0.0", long_about = None)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(short, long, default_value = "6379")]
    port: u16,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    // Redis command
    #[command(subcommand)]
    command: Option<RedisCommand>,
    // command arguments
    #[arg(value_name = "args")]
    args: Vec<String>,
}

/// This enum represents the various commands that can be executed in the CLI.
/// Each variant corresponds to a Redis command and its associated arguments.
#[derive(Subcommand, Debug)]
enum RedisCommand {
    Ping { msg: Option<String> },
    Get { key: String },
    Set { key: String, value: String },
    Del { keys: Vec<String> },
    Exists { keys: Vec<String> },
    Expire,
    Ttl,
    Incr,
    Decr,
    Lpush,
    Rpush,
    Lpop,
    Rpop,
    Lrange,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    // Connect to the Redis server
    let addr = format!("{}:{}", cli.host, cli.port);

    let mut client = Client::connect(&addr).await?;

    if let Some(command) = cli.command {
        match command {
            RedisCommand::Ping { msg } => {
                let response = client.ping(msg.as_deref()).await?;
                println!("{response:?}");
            }
            RedisCommand::Get { key } => {
                let response = client.get(&key).await?;
                if let Some(value) = response {
                    println!("{value:?}");
                } else {
                    println!("(nil)");
                }
            }
            RedisCommand::Set { key, value } => {
                let response = client.set(&key, &value).await?;
                if let Some(value) = response {
                    println!("{value}");
                } else {
                    println!("(nil)");
                }
            }
            RedisCommand::Del { keys } => {
                let response = client
                    .del(keys.iter().map(String::as_str).collect::<Vec<&str>>())
                    .await?;
                println!("{:?}", response);
            }
            _ => {
                eprintln!("Command not implemented yet");
            }
        }
    } else {
        // Interactive mode if no command is provided
        println!("Interactive mode. Type 'exit' to quit.");

        loop {
            print_and_flush(format!("{addr}> ").as_str());

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "exit" {
                break;
            }

            println!("Sending command: {}", input);
        }
    }

    Ok(())
}

// TODO: catch signals like Ctrl+C and Ctrl+D

/// Prints a message to the console and flushes the output.
/// This function is used to ensure that the message is displayed immediately.
/// It is useful for interactive command-line applications where immediate feedback is required.
///
/// # Arguments
///
/// * `msg` - The message to be printed to the console.
fn print_and_flush(msg: &str) {
    print!("{msg}"); // Print the message
    io::stdout().flush().unwrap(); // Flush the buffer
}
