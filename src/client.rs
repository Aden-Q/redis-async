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
use anyhow::{Context, anyhow};
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
        let stream = TcpStream::connect(addr)
            .await
            .with_context(|| "failed to connect to Redis server")?;

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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for HELLO command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for HELLO command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for PING command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for PING command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for GET command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for GET command")?
        {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends a GETEX command to the Redis server.
    ///
    /// # Description
    /// The GETEX command retrieves the value of a key stored on the Redis server and sets an expiry time.
    ///
    /// # Arguments
    ///
    /// * `key` - A required key to send to the server
    /// * `expiry` - An optional expiry time to set
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` if the key to GETEX exists
    /// * `Ok(None)` if the key to GETEX does not exist
    /// * `Err(RedisError)` if an error occurs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use async_redisx::{Client, Expiry};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///     let resp = client.get_ex("mykey", Some(Expirt::EX(1_u64))).await?;
    /// }
    /// ```
    pub async fn get_ex(&mut self, key: &str, expiry: Option<Expiry>) -> Result<Option<Vec<u8>>> {
        let frame: Frame = GetEx::new(key, expiry).into_stream();

        self.conn.write_frame(&frame).await?;

        match self.read_response().await? {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends a MGET command to the Redis server.
    #[allow(unused_variables)]
    pub async fn mget(&mut self, keys: Vec<&str>) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("MGET command is not implemented yet");
        // let frame: Frame = MGet::new(keys).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for SET command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for SET command")?
        {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends a SETEX command to the Redis server.
    #[allow(unused_variables)]
    pub async fn set_ex(&mut self, key: &str, val: &[u8], seconds: i64) -> Result<Option<Vec<u8>>> {
        todo!("SETEX command is not implemented yet");
        // let frame: Frame = SetEx::new(key, val, seconds).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a SETNX command to the Redis server.
    #[allow(unused_variables)]
    pub async fn set_nx(&mut self, key: &str, val: &[u8]) -> Result<Option<Vec<u8>>> {
        todo!("SETNX command is not implemented yet");
        // let frame: Frame = SetNx::new(key, val).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for DEL command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for DEL command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for EXISTS command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for EXISTS command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for EXPIRE command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for EXPIRE command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for TTL command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for TTL command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for INCR command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for INCR command")?
        {
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends an INCRBY command to the Redis server.
    #[allow(unused_variables)]
    pub async fn incr_by(&mut self, key: &str, increment: i64) -> Result<i64> {
        todo!("INCRBY command is not implemented yet");
        // let frame: Frame = IncrBy::new(key, increment).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an INCRBYFLOAT command to the Redis server.
    #[allow(unused_variables)]
    pub async fn incr_by_float(&mut self, key: &str, increment: f64) -> Result<f64> {
        todo!("INCRBYFLOAT command is not implemented yet");
        // let frame: Frame = IncrByFloat::new(key, increment).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(from_utf8(&data)?.parse::<f64>()?),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for DECR command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for DECR command")?
        {
            Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends a DECRBY command to the Redis server.
    #[allow(unused_variables)]
    pub async fn decr_by(&mut self, key: &str, decrement: i64) -> Result<i64> {
        todo!("DECRBY command is not implemented yet");
        // let frame: Frame = DecrBy::new(key, decrement).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(from_utf8(&data)?.parse::<i64>()?),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a DECRBYFLOAT command to the Redis server.
    #[allow(unused_variables)]
    pub async fn decr_by_float(&mut self, key: &str, decrement: f64) -> Result<f64> {
        todo!("DECRBYFLOAT command is not implemented yet");
        // let frame: Frame = DecrByFloat::new(key, decrement).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(from_utf8(&data)?.parse::<f64>()?),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for LPUSH command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for LPUSH command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for RPUSH command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for RPUSH command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for LPOP command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for LPOP command")?
        {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    pub async fn lpop_n(&mut self, key: &str, count: u64) -> Result<Option<Vec<Vec<u8>>>> {
        let frame: Frame = LPop::new(key, Some(count)).into_stream();

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for LPOP command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for LPOP command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for RPOP command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for RPOP command")?
        {
            Response::Simple(data) => Ok(Some(data)),
            Response::Null => Ok(None),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    pub async fn rpop_n(&mut self, key: &str, count: u64) -> Result<Option<Vec<Vec<u8>>>> {
        let frame: Frame = RPop::new(key, Some(count)).into_stream();

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for RPOP command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for RPOP command")?
        {
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

        self.conn
            .write_frame(&frame)
            .await
            .with_context(|| "failed to write frame for LRANGE command")?;

        match self
            .read_response()
            .await
            .with_context(|| "failed to read response for LRANGE command")?
        {
            Response::Array(data) => Ok(data),
            Response::Error(err) => Err(err),
            _ => Err(RedisError::UnexpectedResponseType),
        }
    }

    /// Sends an HGET command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hget(&mut self, key: &str, field: &str) -> Result<Option<Vec<u8>>> {
        todo!("HGET command is not implemented yet");
        // let frame: Frame = HGet::new(key, field).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HMGET command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hmget(&mut self, key: &str, fields: Vec<&str>) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("HMGET command is not implemented yet");
        // let frame: Frame = HMGet::new(key, fields).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HGETALL command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hget_all(&mut self, key: &str) -> Result<Option<HashMap<String, Vec<u8>>>> {
        todo!("HGETALL command is not implemented yet");
        // let frame: Frame = HGetAll::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Map(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HKEYS command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hkeys(&mut self, key: &str) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("HKEYS command is not implemented yet");
        // let frame: Frame = HKeys::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HVALS command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hvals(&mut self, key: &str) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("HVALS command is not implemented yet");
        // let frame: Frame = HVals::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HLEN command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hlen(&mut self, key: &str) -> Result<Option<u64>> {
        todo!("HLEN command is not implemented yet");
        // let frame: Frame = HLen::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<u64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HSET command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hset(&mut self, key: &str, field: &str, value: &[u8]) -> Result<Option<Vec<u8>>> {
        todo!("HSET command is not implemented yet");
        // let frame: Frame = HSet::new(key, field, value).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HSETNX command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hset_nx(
        &mut self,
        key: &str,
        field: &str,
        value: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        todo!("HSETNX command is not implemented yet");
        // let frame: Frame = HSetNx::new(key, field, value).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HMSET command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hmset(
        &mut self,
        key: &str,
        fields: HashMap<String, Vec<u8>>,
    ) -> Result<Option<Vec<u8>>> {
        todo!("HMSET command is not implemented yet");
        // let frame: Frame = HMSet::new(key, fields).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an HDEL command to the Redis server.
    #[allow(unused_variables)]
    pub async fn hdel(&mut self, key: &str, field: &str) -> Result<Option<Vec<u8>>> {
        todo!("HDEL command is not implemented yet");
        // let frame: Frame = HDel::new(key, field).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an SADD command to the Redis server.
    #[allow(unused_variables)]
    pub async fn sadd(&mut self, key: &str, members: Vec<&[u8]>) -> Result<Option<Vec<u8>>> {
        todo!("SADD command is not implemented yet");
        // let frame: Frame = SAdd::new(key, members).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an SREM command to the Redis server.
    #[allow(unused_variables)]
    pub async fn srem(&mut self, key: &str, members: Vec<&[u8]>) -> Result<Option<Vec<u8>>> {
        todo!("SREM command is not implemented yet");
        // let frame: Frame = SRem::new(key, members).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an SISMEMBER command to the Redis server.
    #[allow(unused_variables)]
    pub async fn sismember(&mut self, key: &str, member: &[u8]) -> Result<Option<Vec<u8>>> {
        todo!("SISMEMBER command is not implemented yet");
        // let frame: Frame = SIsMember::new(key, member).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an SMEMBERS command to the Redis server.
    #[allow(unused_variables)]
    pub async fn smembers(&mut self, key: &str) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("SMEMBERS command is not implemented yet");
        // let frame: Frame = SMembers::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends an SPOP command to the Redis server.
    #[allow(unused_variables)]
    pub async fn spop(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        todo!("SPOP command is not implemented yet");
        // let frame: Frame = SPop::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZADD command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zadd(
        &mut self,
        key: &str,
        members: HashMap<String, f64>,
    ) -> Result<Option<Vec<u8>>> {
        todo!("ZADD command is not implemented yet");
        // let frame: Frame = ZAdd::new(key, members).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZREM command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zrem(&mut self, key: &str, members: Vec<&[u8]>) -> Result<Option<Vec<u8>>> {
        todo!("ZREM command is not implemented yet");
        // let frame: Frame = ZRem::new(key, members).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZRANGE command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zrange(
        &mut self,
        key: &str,
        start: i64,
        end: i64,
    ) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("ZRANGE command is not implemented yet");
        // let frame: Frame = ZRange::new(key, start, end).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZREVRANGE command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zrevrange(
        &mut self,
        key: &str,
        start: i64,
        end: i64,
    ) -> Result<Option<Vec<Vec<u8>>>> {
        todo!("ZREVRANGE command is not implemented yet");
        // let frame: Frame = ZRevRange::new(key, start, end).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Array(data) => Ok(Some(data)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZRANK command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zrank(&mut self, key: &str, member: &[u8]) -> Result<Option<u64>> {
        todo!("ZRANK command is not implemented yet");
        // let frame: Frame = ZRank::new(key, member).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<u64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZREVRANK command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zrevrank(&mut self, key: &str, member: &[u8]) -> Result<Option<u64>> {
        todo!("ZREVRANK command is not implemented yet");
        // let frame: Frame = ZRevRank::new(key, member).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<u64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZSCORE command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zscore(&mut self, key: &str, member: &[u8]) -> Result<Option<f64>> {
        todo!("ZSCORE command is not implemented yet");
        // let frame: Frame = ZScore::new(key, member).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<f64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZCARD command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zcard(&mut self, key: &str) -> Result<Option<u64>> {
        todo!("ZCARD command is not implemented yet");
        // let frame: Frame = ZCard::new(key).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<u64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZCOUNT command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zcount(&mut self, key: &str, min: f64, max: f64) -> Result<Option<u64>> {
        todo!("ZCOUNT command is not implemented yet");
        // let frame: Frame = ZCount::new(key, min, max).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<u64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
    }

    /// Sends a ZINCRBY command to the Redis server.
    #[allow(unused_variables)]
    pub async fn zincr_by(
        &mut self,
        key: &str,
        increment: f64,
        member: &[u8],
    ) -> Result<Option<f64>> {
        todo!("ZINCRBY command is not implemented yet");
        // let frame: Frame = ZIncrBy::new(key, increment, member).into_stream();

        // self.conn.write_frame(&frame).await?;

        // match self.read_response().await? {
        //     Response::Simple(data) => Ok(Some(from_utf8(&data)?.parse::<f64>()?)),
        //     Response::Null => Ok(None),
        //     Response::Error(err) => Err(err),
        //     _ => Err(RedisError::UnexpectedResponseType),
        // }
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
