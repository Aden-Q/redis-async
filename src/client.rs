//! Redis client CLI application. A simple command line interface to interact with a Redis server.
//!
//! The clients default to RESP2 unless HELLO 3 is explicitly sent.
//! It can operate in two modes: interactive and single command mode.
//! In interactive mode, the user can send commands to the server and get the response. It starts an REPL loop.
//! In single command mode, the user can send a single command to the server and get the response.
//! Both modes are blocking and synchronous.

use crate::Connection;
use crate::Frame;
use crate::RedisError;
use crate::Result;
use crate::cmd::*;
use crate::error::wrap_error;
use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};

/// Redis client implementation.
pub struct Client {
    // todo: modify it to use a connection pool shared across multiple clients
    // spawn a new connection for each client is inefficient when the number of clients is large
    conn: Connection,
}

impl Client {
    /// Establish a connection to the Redis server.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use async_redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut c = Client::connect("127.0.0.1:6379").await.unwrap();
    /// }
    /// ```
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await.map_err(wrap_error)?;

        let conn = Connection::new(stream);

        Ok(Client { conn })
    }

    /// Sends a PING command to the Redis server, optionally with a message.
    ///
    /// # Arguments
    ///
    /// * `msg` - An optional message to send to the server
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the PING command is successful
    /// * `Err(RedisError)` if an error occurs
    ///     
    /// # Examples
    ///
    /// ```ignore
    /// use async_redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    /// let resp = client.ping(Some("Hello Redis".to_string())).await.unwrap();
    /// ```
    pub async fn ping(&mut self, msg: Option<&str>) -> Result<String> {
        let frame: Frame = Ping::new(msg).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => {
                let resp = String::from_utf8(data.to_vec()).unwrap();
                Ok(resp)
            }
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends a GET command to the Redis server, with a key.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to send to the server
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` if the key to GET exists
    /// * `Ok(None)` if the key to GET does not exist
    /// * `Err(RedisError)` if an error occurs
    ///     
    /// # Examples
    ///
    /// ```ignore
    /// use async_redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    /// let resp = client.get("mykey").await?;
    /// ```
    pub async fn get(&mut self, key: &str) -> Result<Option<String>> {
        let frame: Frame = Get::new(key).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => {
                let resp = String::from_utf8(data.to_vec()).unwrap();
                Ok(Some(resp))
            }
            // no error, but the key doesn't exist
            None => Ok(None),
        }
    }

    // todo: the real SET command has some other options like EX, PX, NX, XX
    // we need to add these options to the SET command. Possibly with option pattern
    pub async fn set(&mut self, key: &str, val: &str) -> Result<Option<String>> {
        let frame: Frame = Set::new(key, val).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => {
                let resp = String::from_utf8(data.to_vec()).unwrap();
                Ok(Some(resp))
            }
            // no error, but the key doesn't exist
            None => Ok(None),
        }
    }

    /// Reads the response from the server. The response is a searilzied frame.
    /// It decodes the frame and returns the human readable message to the client.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Bytes))` if the response is successfully read
    /// * `Ok(None)` if the response is empty
    /// * `Err(RedisError)` if an error occurs
    async fn read_response(&mut self) -> Result<Option<Bytes>> {
        match self.conn.read_frame().await? {
            Some(Frame::SimpleString(data)) => Ok(Some(Bytes::from(data))),
            Some(Frame::SimpleError(data)) => Err(wrap_error(RedisError::Other(data.into()))),
            Some(Frame::BulkString(data)) => Ok(Some(data)),
            Some(Frame::Null) => Ok(None),
            Some(_) => unimplemented!(),
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }
}
