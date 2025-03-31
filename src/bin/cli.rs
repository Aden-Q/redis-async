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

use bytes::Bytes;
use clap::{Parser, Subcommand};
use colored::Colorize;
use redis_async::{Client, Result};
use shlex::split;
use std::io::{self, Write};
use std::str;

#[derive(Parser, Debug)]
#[command(name = "redis-async-cli")]
#[command(version = "0.1.0")]
#[command(about = "redis-cli 0.1.0", long_about = None)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1", help = "Redis server hostname.")]
    host: String,
    #[arg(short, long, default_value = "6379", help = "Redis server port.")]
    port: u16,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    // Redis command
    #[command(subcommand)]
    command: Option<RedisCommand>,
}

#[derive(Parser, Debug)]
struct CliInteractive {
    // Redis command
    #[command(subcommand)]
    command: Option<RedisCommand>,
}

/// This enum represents the various commands that can be executed in the CLI.
/// Each variant corresponds to a Redis command and its associated arguments.
#[derive(Subcommand, Debug, Clone)]
enum RedisCommand {
    /// Switch RESP protocol version.
    Hello {
        /// RESP protocol version to switch to.
        proto: Option<u8>,
    },
    /// Check if the server is alive.
    Ping {
        /// Message to send to the server.
        message: Option<Bytes>,
    },
    /// Get the value of a key.
    Get {
        /// Key to retrieve.
        key: String,
    },
    /// Set the value of a key.
    Set {
        /// Key to set.
        key: String,
        /// Value to set.
        value: Bytes,
    },
    /// Delete a key.
    Del {
        /// Keys to delete.
        keys: Vec<String>,
    },
    /// Check if a key exists.
    Exists {
        /// Keys to check.
        keys: Vec<String>,
    },
    /// Expire a key after a given number of seconds.
    Expire {
        /// Key to expire.
        key: String,
        /// Number of seconds to expire the key after.
        seconds: i64,
    },
    /// Get the time to live of a key.
    Ttl {
        /// Key to check.
        key: String,
    },
    /// Increment the value of a key.
    Incr {
        /// Key to increment.
        key: String,
    },
    /// Decrement the value of a key.
    Decr {
        /// Key to decrement.
        key: String,
    },
    /// Push a value onto a list. Left push.
    Lpush {
        /// Key of the list.
        key: String,
        /// Values to push onto the list.
        values: Vec<String>,
    },
    /// Push a value onto a list. Right push.
    Rpush {
        /// Key of the list.
        key: String,
        /// Values to push onto the list.
        values: Vec<String>,
    },
    /// Pop values from a list. Left pop.
    Lpop {
        /// Key of the list.
        key: String,
        /// Number of elements to pop.
        /// If not specified, it will pop only one element.
        count: Option<u64>,
    },
    /// Pop values from a list. Right pop.
    Rpop {
        /// Key of the list.
        key: String,
        /// Number of elements to pop.
        /// If not specified, it will pop only one element.
        count: Option<u64>,
    },
    /// Get a range of values from a list.
    Lrange {
        /// Key of the list.
        key: String,
        /// Start index of the range.
        start: i64,
        /// End index of the range.
        end: i64,
    },
    /// Clear the screen.
    Clear,
}

impl RedisCommand {
    async fn execute(&self, client: &mut Client) -> Result<()> {
        match self {
            RedisCommand::Hello { proto } => {
                let response = client.hello(*proto).await?;
                if let Ok(string) = str::from_utf8(&response) {
                    println!("\"{}\"", string);
                } else {
                    println!("{response:?}");
                }
            }
            RedisCommand::Ping { message } => {
                let message = message.as_deref();

                let response = client.ping(message).await?;
                if let Ok(string) = str::from_utf8(&response) {
                    // we need to format simple string and bulk string differently
                    // simple string: no quotes
                    // bulk string: with quotes
                    if message.is_some() {
                        println!("\"{}\"", string);
                    } else {
                        println!("PONG");
                    }
                } else {
                    println!("{response:?}");
                }
            }
            RedisCommand::Get { key } => {
                let response = client.get(key).await?;
                if let Some(value) = response {
                    if let Ok(string) = str::from_utf8(&value) {
                        println!("\"{}\"", string);
                    } else {
                        println!("{:?}", value);
                    }
                } else {
                    println!("(nil)");
                }
            }
            RedisCommand::Set { key, value } => {
                let response = client.set(key, value).await?;
                if let Some(value) = response {
                    if let Ok(string) = str::from_utf8(&value) {
                        println!("{}", string);
                    } else {
                        println!("{:?}", value);
                    }
                } else {
                    println!("(nil)");
                }
            }
            RedisCommand::Del { keys } => {
                let response = client
                    .del(keys.iter().map(String::as_str).collect::<Vec<&str>>())
                    .await?;
                println!("{response:?}");
            }
            RedisCommand::Exists { keys } => {
                let response = client
                    .exists(keys.iter().map(String::as_str).collect::<Vec<&str>>())
                    .await?;
                println!("(integer) {response}");
            }
            RedisCommand::Expire { key, seconds } => {
                let response = client.expire(key, *seconds).await?;
                println!("(integer) {response}");
            }
            RedisCommand::Ttl { key } => {
                let response = client.ttl(key).await?;
                println!("(integer) {response}");
            }
            RedisCommand::Incr { key } => {
                let response = client.incr(key).await?;
                println!("(integer) {response}");
            }
            RedisCommand::Decr { key } => {
                let response = client.decr(key).await?;
                println!("(integer) {response}");
            }
            RedisCommand::Lpush { key, values } => {
                let response = client
                    .lpush(key, values.iter().map(String::as_str).collect())
                    .await?;
                println!("(integer) {response}");
            }
            RedisCommand::Rpush { key, values } => {
                let response = client
                    .rpush(key, values.iter().map(String::as_str).collect())
                    .await?;
                println!("(integer) {response}");
            }
            RedisCommand::Lpop { key, count } => {
                let response = client.lpop(key, *count).await?;
                println!("{response:?}");
            }
            RedisCommand::Rpop { key, count } => {
                let response = client.rpop(key, *count).await?;
                println!("{response:?}");
            }
            RedisCommand::Lrange { key, start, end } => {
                let response = client.lrange(key, *start, *end).await?;
                println!("{response:?}");
            }
            RedisCommand::Clear => {
                clear_screen();
            }
        }

        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Collect raw arguments and normalize subcommands to lowercase
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        args[1] = args[1].to_lowercase(); // Normalize the subcommand
    }

    let cli = Cli::parse_from(&args);

    // Set up the address for the Redis server
    let mut addr = String::with_capacity(cli.host.len() + 1 + cli.port.to_string().len());
    addr.push_str(&cli.host);
    addr.push(':');
    addr.push_str(&cli.port.to_string());

    // Connect to the Redis server
    let mut client = Client::connect(&addr).await?;

    if let Some(command) = cli.command {
        // If a command is provided, execute it
        command.execute(&mut client).await?;
    } else {
        // Interactive mode if no command is provided
        println!("{}", "Interactive mode. Type 'exit' to quit.".green());

        loop {
            print!("{addr}> "); // Print the prompt
            io::stdout().flush().unwrap(); // Flush the buffer

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "exit" {
                break;
            }

            let args = split(input).unwrap();
            if args.is_empty() {
                continue;
            }

            // Convert the first argument to lowercase
            let mut args = args.to_vec();
            let lowercased = args[0].to_lowercase();
            args[0] = lowercased;

            // we need to insert the command name at the beginning of the args vector
            // otherwise clap parser will not be able to parse the command
            args.insert(0, "".into());

            match CliInteractive::try_parse_from(args) {
                Ok(cli) => {
                    // If a command is provided, execute it
                    if let Some(command) = cli.command {
                        match command.execute(&mut client).await {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error executing command: {e}");
                                // do not fail the program, just continue
                                continue;
                            }
                        }
                    } else {
                        println!("Unknown command: {input}");
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing command: {e}");
                    // do not fail the program, just continue
                    continue;
                }
            };
        }
    }

    Ok(())
}

// TODO: catch signals like Ctrl+C and Ctrl+D
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H"); // Clears the screen and moves the cursor to the top-left
    std::io::stdout().flush().unwrap();
}
