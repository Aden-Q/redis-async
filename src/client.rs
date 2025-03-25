use crate::Connection;
use crate::Frame;
use crate::Result;
use crate::cmd::{Command, Ping};
use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    // todo: modify it to use a connection pool
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
        let stream = TcpStream::connect(addr).await?;

        let conn = Connection::new(stream);

        Ok(Client { conn })
    }

    pub async fn ping(&mut self, msg: Option<String>) -> Result<String> {
        let frame: Frame = Ping::new(msg.unwrap_or_default()).into_stream();

        self.conn.write_frame(&frame).await?;

        // todo: read response from the server and return to the client
        match self.read_response().await? {
            Some(data) => {
                let resp = String::from_utf8(data.to_vec()).unwrap();
                Ok(resp)
            }
            None => Err("Redis error".into()),
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

    async fn read_response(&mut self) -> Result<Option<Bytes>> {
        match self.conn.read_frame().await? {
            Some(Frame::SimpleString(data)) => Ok(Some(Bytes::from(data))),
            Some(Frame::SimpleError(data)) => Err(data.into()),
            _ => Err("protocol error".into()),
        }
    }
}
