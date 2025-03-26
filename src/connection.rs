use crate::Frame;
use crate::RedisError;
use crate::Result;
use crate::error::wrap_error;
use bytes::{Bytes, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

/// Represents a connection bewteen the client and the Redis server.
///
/// The connecton wraps a TCP stream and a buffer for reading and writing Frames.
///
/// To read Frames, the connection waits asynchronously until there is enough data to parse a Frame.
/// On success, it deserializes the bytes into a Frame and returns it to the client.
///
/// To write Frames, the connection serializes the Frame into bytes and writes it to the stream.
/// It then flushes the stream to ensure the data is sent to the server.
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    /// Creates a new connection from a TCP stream. The stream is wrapped in a write buffer.
    /// It also initializes a read buffer for reading from the TCP stream. The read buffer is 4kb.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            // 4kb buffer for each connection
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Reads a single Redis Frame from the TCP stream.
    ///
    /// The method reads from the stream into the buffer until it has a complete Frame.
    /// It then parses the Frame and returns it to the client.
    ///
    /// # Returns
    ///
    /// An Option containing the Frame if it was successfully read and parsed.
    /// None if the Frame is incomplete and more data is needed.
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.try_parse_frame().await? {
                return Ok(Some(frame));
            }

            // read from the stream into the buffer until we have a frame
            if 0 == self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .map_err(wrap_error)?
            {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(wrap_error(RedisError::Other("Unknown error".to_string())));
                }
            }
        }
    }

    /// Writes a single Redis Frame to the TCP stream.
    ///
    /// The method serializes the Frame into bytes and writes it to the stream.
    /// It then flushes the stream to ensure the data is sent to the server.
    ///
    /// # Arguments
    ///
    /// * `frame` - A reference to the Frame to be written to the stream
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        let bytes: Bytes = frame.serialize().await?;

        self.stream.write_all(&bytes).await.map_err(wrap_error)?;

        self.stream.flush().await.map_err(wrap_error)?;

        Ok(())
    }

    /// Tries to parse a single Redis Frame from the buffer.
    ///
    /// The method checks if the buffer contains a complete Frame.
    /// If it does, it deserializes the bytes into a Frame and returns it to the client.
    /// If the Frame is incomplete, it returns None.
    ///
    /// # Returns
    ///
    /// An Option containing the Frame if it was successfully read and parsed.
    /// None if the Frame is incomplete and more data is needed.
    /// An error if the Frame is invalid.
    async fn try_parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf: Cursor<&[u8]> = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf).await {
            // Ok means we can parse a complete frame
            Ok(()) => {
                let len = buf.position() as usize;

                let bytes = self.buffer.split_to(len).freeze();

                // once we have read the frame, we can advance the buffer
                println!("try_parse_frame: len={len}, bytes={bytes:?}");

                Ok(Some(Frame::deserialize(bytes).await?))
            }
            Err(err) => match &*err {
                // IncompleteFrame means we need to read more data
                RedisError::IncompleteFrame => Ok(None),
                _ => Err(err),
            },
        }
    }
}
