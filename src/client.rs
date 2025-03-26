use crate::Connection;
use crate::Frame;
use crate::RedisError;
use crate::Result;
use crate::cmd::{Command, Ping};
use crate::error::wrap_error;
use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    // todo: modify it to use a connection pool shared across multiple clients
    // spawn a new connection for each client is inefficient when the number of clients is large
    conn: Connection,
}

impl Client {
    /// Establish a connection to the Redis server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// ```
    ///
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await.map_err(wrap_error)?;

        let conn = Connection::new(stream);

        Ok(Client { conn })
    }

    pub async fn ping(&mut self, msg: Option<String>) -> Result<String> {
        let frame: Frame = Ping::new(msg).into_stream();

        self.conn.write_frame(&frame).await?;

        // todo: read response from the server and return to the client
        match self.read_response().await? {
            Some(data) => {
                let resp = String::from_utf8(data.to_vec()).unwrap();
                Ok(resp)
            }
            None => Err(wrap_error(RedisError::Other("Unknown error".to_string()))),
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

    /// read a response from the server
    /// decode the frame and return the meaning message to the client
    async fn read_response(&mut self) -> Result<Option<Bytes>> {
        match self.conn.read_frame().await? {
            Some(Frame::SimpleString(data)) => Ok(Some(Bytes::from(data))),
            Some(Frame::SimpleError(data)) => Err(wrap_error(RedisError::Other(data))),
            Some(Frame::BulkString(data)) => Ok(Some(Bytes::from(data))),
            Some(_) => Err(wrap_error(RedisError::Other(
                "Unknown frame type: not implemented".to_string(),
            ))),
            _ => Err(wrap_error(RedisError::Other(
                "Error reading frame".to_string(),
            ))),
        }
    }
}
