use crate::Connection;
use crate::Frame;
use crate::RedisError;
use crate::Result;
use crate::cmd::{Command, Ping};
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
    /// * `msg` - An optional message to send to the server.
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
    ///     let mut c = Client::connect("127.0.0.1:6379").await.unwrap();
    ///
    ///     let resp = c.ping(Some("Hello Redis".to_string())).await.unwrap();
    /// }
    pub async fn ping(&mut self, msg: Option<String>) -> Result<String> {
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

    #[allow(dead_code)]
    pub async fn get(&self, _: &str) -> Self {
        unimplemented!()
    }

    #[allow(dead_code)]
    pub async fn set(&self, _: &str, _: String) -> Self {
        unimplemented!()
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
            Some(_) => unimplemented!(),
            None => Err(wrap_error(RedisError::Other("Unknown error".into()))),
        }
    }
}
