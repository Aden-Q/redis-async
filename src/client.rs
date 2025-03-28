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
use std::str::from_utf8;
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
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.ping(Some("Hello Redis".to_string())).await.unwrap();
    /// }
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

    /// Sends a GET command to the Redis server.
    ///
    /// # Description
    ///
    /// The GET command retrieves the value of a key stored on the Redis server.
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
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.get("mykey").await?;
    /// }
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
    /// Sends a SET command to the Redis server.
    ///
    /// # Description
    ///
    /// The SET command sets the value of a key in the Redis server.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to set
    /// * `val` - A required value to set
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` if the key is set successfully
    /// * `Ok(None)` if the key is not set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use async_redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.set("mykey", "myvalue").await?;
    /// }
    pub async fn set(&mut self, key: &str, val: &str) -> Result<Option<String>> {
        let frame: Frame = Set::new(key, val).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => {
                let resp = String::from_utf8(data.to_vec()).unwrap();
                Ok(Some(resp))
            }
            // we shouldn't get here, if no key is deleted, we expect an 0
            None => Ok(None),
        }
    }

    /// Sends a DEL command to the Redis server.
    ///
    /// # Description
    ///
    /// The DEL command deletes a key from the Redis server.
    ///
    /// # Arguments
    ///
    /// * `keys` - A required vector of keys to delete
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` the number of keys deleted
    ///
    /// # Examples
    ///
    /// ```ignore
    ///
    /// use async_redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.del(vec!["foo", "bar", "baz"]).await?;
    /// }
    pub async fn del(&mut self, keys: Vec<&str>) -> Result<u64> {
        let frame: Frame = Del::new(keys).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends an EXISTS command to the Redis server.
    ///
    /// # Description
    ///
    /// The EXISTS command checks if a key exists in the Redis server.
    ///
    /// # Arguments
    ///
    /// * `keys` - A required vector of keys to check
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` the number of keys that exist
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.exists(vec!["foo", "bar", "baz"]).await?;
    /// }
    pub async fn exists(&mut self, keys: Vec<&str>) -> Result<u64> {
        let frame: Frame = Exists::new(keys).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    // todo: add EXAT, PXAT, NX, XX options
    /// Sends an EXPIRE command to the Redis server.
    ///
    /// # Description
    ///
    /// The EXPIRE command sets a timeout on a key. After the timeout has expired, the key will be deleted.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to set the timeout
    /// * `seconds` - A required number of seconds to set the timeout
    ///
    /// # Returns
    ///
    /// * `Ok(1)` if the key is set successfully
    /// * `Ok(0)` if the key is not set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.expire("mykey", 1).await?;
    /// }
    pub async fn expire(&mut self, key: &str, seconds: i64) -> Result<u64> {
        let frame: Frame = Expire::new(key, seconds).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends a TTL command to the Redis server.
    ///
    /// # Description
    ///
    /// The TTL command returns the remaining time to live of a key that has an expire set.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to check ttl
    ///
    /// # Returns
    ///
    /// * `Ok(-2)` if the key does not exist
    /// * `Ok(-1)` if the key exists but has no expire set
    /// * `Ok(other)` if the key exists and has an expire set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.ttl("mykey").await?;
    /// }
    pub async fn ttl(&mut self, key: &str) -> Result<i64> {
        let frame: Frame = Ttl::new(key).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            // we shouldn't get here, we alawys expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends an INCR command to the Redis server.
    ///
    /// # Description
    ///
    /// The INCR command increments the integer value of a key by one.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to increment
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` the new value of the key after increment
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.incr("mykey").await?;
    /// }
    pub async fn incr(&mut self, key: &str) -> Result<i64> {
        let frame: Frame = Incr::new(key).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends a DECR command to the Redis server.
    ///
    /// # Description
    ///
    /// The DECR command decrements the integer value of a key by one.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to decrement
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` the new value of the key after decrement
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.decr("mykey").await?;
    /// }
    pub async fn decr(&mut self, key: &str) -> Result<i64> {
        let frame: Frame = Decr::new(key).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends an LPUSH command to the Redis server.
    ///
    /// # Description
    ///
    /// The LPUSH command inserts all the specified values at the head of the list stored at key.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to insert values
    /// * `values` - A required vector of values to insert
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` the length of the list after the push operation
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.lpush("mykey", vec!["foo", "bar", "baz"]).await?;
    /// }
    pub async fn lpush(&mut self, key: &str, values: Vec<&str>) -> Result<u64> {
        let frame: Frame = LPush::new(key, values).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends an RPUSH command to the Redis server.
    ///
    /// # Description
    ///
    /// The RPUSH command inserts all the specified values at the tail of the list stored at key.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to insert values
    /// * `values` - A required vector of values to insert
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` the length of the list after the push operation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.rpush("mykey", vec!["foo", "bar", "baz"]).await?;
    /// }
    pub async fn rpush(&mut self, key: &str, values: Vec<&str>) -> Result<u64> {
        let frame: Frame = RPush::new(key, values).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Some(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            // we shouldn't get here, we always expect a number from the server
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }

    /// Sends an LPOP command to the Redis server.
    ///
    /// # Description
    ///
    /// The LPOP command removes and returns the removed elements from the head of the list stored at key.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to remove values
    /// * `count` - A required number of elements to remove
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` if the key exists and the elements are removed
    /// * `Ok(None)` if the key does not exist
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.lpop("mykey", 1).await?;
    /// }
    pub async fn lpop(&mut self, key: &str, count: u64) -> Result<Option<String>> {
        let frame: Frame = LPop::new(key, count).into_stream();

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

    /// Sends an RPOP command to the Redis server.
    ///
    /// # Description
    ///
    /// The RPOP command removes and returns the removed elements from the tail of the list stored at key.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to remove values
    /// * `count` - A required number of elements to remove
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` if the key exists and the elements are removed
    /// * `Ok(None)` if the key does not exist
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.rpop("mykey", 1).await?;
    /// }
    pub async fn rpop(&mut self, key: &str, count: u64) -> Result<Option<String>> {
        let frame: Frame = RPop::new(key, count).into_stream();

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

    pub async fn lrange(&mut self, key: &str, start: i64, end: i64) -> Result<Option<String>> {
        let frame: Frame = LRange::new(key, start, end).into_stream();

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
            Some(Frame::Integer(data)) => Ok(Some(Bytes::from(data.to_string()))),
            Some(Frame::BulkString(data)) => Ok(Some(data)),
            Some(Frame::Array(data)) => {
                let result = data
                    .into_iter()
                    .map(|frame| match frame {
                        Frame::BulkString(data) => Ok(data),
                        Frame::SimpleString(data) => Ok(Bytes::from(data)),
                        Frame::Integer(data) => Ok(Bytes::from(data.to_string())),
                        _ => Err(wrap_error(RedisError::InvalidFrame)),
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Some(Bytes::from(result.concat())))
            }
            Some(Frame::Null) => Ok(None), // nil reply usually means no error
            // todo: array response needed here
            Some(_) => unimplemented!(),
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }
}
