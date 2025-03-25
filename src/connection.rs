use crate::Frame;
use crate::Result;
use bytes::Buf;
use bytes::{Bytes, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            // 4kb buffer for each connection
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Read a Redis frame from the connection
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.try_parse_frame().await? {
                return Ok(Some(frame));
            }

            // read from the stream into the buffer until we have a frame
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// Write a frame to the connection
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        let bytes: Bytes = frame.serialize().await?;

        println!("write_frame: bytes={bytes:?}");

        self.stream.write_all(&bytes).await?;

        self.stream.flush().await?;

        Ok(())
    }

    async fn try_parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf: Cursor<&[u8]> = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            // Ok means we can parse a complete frame
            Ok(()) => {
                let len = buf.position() as usize;

                let bytes: Bytes = Bytes::copy_from_slice(&self.buffer[..len]);

                // once we have read the frame, we can advance the buffer
                dbg!("try_parse_frame: len={len}, bytes={bytes:?}");

                self.buffer.advance(len);

                Ok(Some(Frame::deserialize(bytes).await?))
            }
            // Err(Incomplete) => Ok(None),
            Err(e) => Ok(None),
        }
    }
}
