//! Redis client implementation.
//!
//! The clients default to RESP2 unless HELLO 3 is explicitly sent to switch to RESP3.
//! The client is a simple wrapper around the Connection struct.
//! It provides simple APIs to send commands to the Redis server and get the response.
//! The client is designed to be used in an async context, using the tokio runtime.

use crate::Connection;
use crate::Frame;
use crate::RedisError;
use crate::Result;
use crate::cmd::*;
use anyhow::anyhow;
use std::collections::HashMap;
use std::str::from_utf8;
use tokio::net::{TcpStream, ToSocketAddrs};

#[derive(Debug)]
pub enum Response {
    Simple(Vec<u8>),
    Array(Vec<Vec<u8>>),
    Map(HashMap<String, Vec<u8>>),
    Null,
    Error(RedisError),
}

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
        let stream = TcpStream::connect(addr).await?;

        let conn = Connection::new(stream);

        Ok(Client { conn })
    }

    /// Sends a HELLO command to the Redis server.
    ///
    /// # Arguments
    ///
    /// * `proto` - An optional protocol version to use
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, Vec<u8>>)` if the HELLO command is successful
    /// * `Err(RedisError)` if an error occurs
    pub async fn hello(&mut self, proto: Option<u8>) -> Result<HashMap<String, Vec<u8>>> {
        let frame: Frame = Hello::new(proto).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Array(data) => {
                let map = data
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            let key = from_utf8(&chunk[0]).ok()?.to_string();
                            let value = chunk[1].to_vec();
                            Some((key, value))
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(map)
            }
            Response::Map(data) => Ok(data),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
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
    pub async fn ping(&mut self, msg: Option<&[u8]>) -> Result<Vec<u8>> {
        let frame: Frame = Ping::new(msg).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(data),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    pub async fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let frame: Frame = Get::new(key).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    pub async fn set(&mut self, key: &str, val: &[u8]) -> Result<Option<Vec<u8>>> {
        let frame: Frame = Set::new(key, val).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    pub async fn lpush(&mut self, key: &str, values: Vec<&[u8]>) -> Result<u64> {
        let frame: Frame = LPush::new(key, values).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    pub async fn rpush(&mut self, key: &str, values: Vec<&[u8]>) -> Result<u64> {
        let frame: Frame = RPush::new(key, values).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<u64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    /// * `count` - An optional number of elements to remove
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
    pub async fn lpop(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let frame: Frame = LPop::new(key, None).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    pub async fn lpop_n(&mut self, key: &str, count: u64) -> Result<Option<Vec<Vec<u8>>>> {
        let frame: Frame = LPop::new(key, Some(count)).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Array(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    /// * `count` - An optional number of elements to remove
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
    pub async fn rpop(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let frame: Frame = RPop::new(key, None).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    pub async fn rpop_n(&mut self, key: &str, count: u64) -> Result<Option<Vec<Vec<u8>>>> {
        let frame: Frame = RPop::new(key, Some(count)).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Array(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends an LRANGE command to the Redis server.
    ///
    /// # Description
    ///
    /// The LRANGE command returns the specified elements of the list stored at key.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to get values
    /// * `start` - A required start index
    /// * `end` - A required end index
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` if the key exists and the elements are returned
    /// * `Ok(None)` if the key does not exist
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.lrange("mykey", 0, -1).await?;
    /// }
    pub async fn lrange(&mut self, key: &str, start: i64, end: i64) -> Result<Vec<Vec<u8>>> {
        let frame: Frame = LRange::new(key, start, end).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Array(data) => Ok(data),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
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
    async fn read_response(&mut self) -> Result<Response> {
        match self.conn.read_frame().await? {
            Some(Frame::SimpleString(data)) => Ok(Response::Simple(data.into_bytes())),
            Some(Frame::SimpleError(data)) => Ok(Response::Error(RedisError::Other(anyhow!(data)))),
            Some(Frame::Integer(data)) => Ok(Response::Simple(data.to_string().into_bytes())),
            Some(Frame::BulkString(data)) => Ok(Response::Simple(data.to_vec())),
            Some(Frame::Array(data)) => {
                let result: Vec<Vec<u8>> = data
                    .into_iter()
                    .map(|frame| match frame {
                        Frame::BulkString(data) => data.to_vec(),
                        Frame::SimpleString(data) => data.into_bytes(),
                        Frame::Integer(data) => data.to_string().into_bytes(),
                        Frame::Array(data) => {
                            let result = data
                                .into_iter()
                                .map(|frame| match frame {
                                    Frame::BulkString(data) => data.to_vec(),
                                    Frame::SimpleString(data) => data.into_bytes(),
                                    Frame::Integer(data) => data.to_string().into_bytes(),
                                    Frame::Null => vec![],
                                    _ => {
                                        vec![]
                                    }
                                })
                                .collect::<Vec<_>>();
                            result.concat()
                        }
                        Frame::Null => vec![],
                        _ => vec![],
                    })
                    .collect();

                Ok(Response::Array(result))
            }
            Some(Frame::Null) => Ok(Response::Null), // nil reply usually means no error
            Some(Frame::Boolean(data)) => {
                if data {
                    Ok(Response::Simple("true".into()))
                } else {
                    Ok(Response::Simple("false".into()))
                }
            }
            Some(Frame::Double(data)) => Ok(Response::Simple(data.to_string().into_bytes())),
            Some(Frame::BulkError(data)) => Ok(Response::Error(RedisError::Other(anyhow!(
                String::from_utf8_lossy(&data).to_string()
            )))),
            Some(Frame::Map(data)) => {
                let result: HashMap<String, Vec<u8>> = data
                    .into_iter()
                    .filter_map(|(key, value)| {
                        let key = match key {
                            Frame::BulkString(data) => String::from_utf8(data.to_vec()).ok(),
                            Frame::SimpleString(data) => Some(data),
                            Frame::Integer(data) => Some(data.to_string()),
                            _ => None,
                        };

                        let value = match value {
                            Frame::BulkString(data) => Some(data.to_vec()),
                            Frame::SimpleString(data) => Some(data.into_bytes()),
                            Frame::Integer(data) => Some(data.to_string().into_bytes()),
                            _ => None,
                        };

                        match (key, value) {
                            (Some(k), Some(v)) => Some((k, v)),
                            _ => None,
                        }
                    })
                    .collect();

                Ok(Response::Map(result))
            }
            // todo: array response needed here
            Some(_) => unimplemented!(""),
            None => Err(RedisError::Unknown),
        }
    }
}
