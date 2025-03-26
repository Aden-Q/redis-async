//! Implements the [RESP3](https://redis.io/docs/latest/develop/reference/protocol-spec)
//! serialization protocol for Redis client-server communication.

use crate::{RedisError, Result, error::wrap_error};
use bytes::{Buf, Bytes, BytesMut};
use std::io::BufRead;

// Frame represents a single RESP frame to transmit over the socket.
#[derive(Debug, PartialEq)]
pub enum Frame {
    // RESP data types
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<Frame>),
    Null,
    Boolean(bool),
}

// we need to implemenet a custom serialization protocol
// for Frame type
impl Frame {
    pub const fn array() -> Self {
        Frame::Array(Vec::new())
    }

    pub fn push_bulk_str(&mut self, item: String) {
        match self {
            Frame::Array(vec) => vec.push(Frame::BulkString(item)),
            _ => unimplemented!(),
        }
    }

    /// method to serialize a Frame into a bytes buffer
    pub async fn serialize(&self) -> Result<Bytes> {
        match self {
            Frame::SimpleString(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 3);

                buf.extend_from_slice(b"+");
                buf.extend_from_slice(val.as_bytes());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::SimpleError(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 3);

                buf.extend_from_slice(b"-");
                buf.extend_from_slice(val.as_bytes());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::BulkString(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 5);

                buf.extend_from_slice(b"$");
                buf.extend_from_slice(val.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                buf.extend_from_slice(val.as_bytes());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::Array(frame_vec) => {
                let mut buf = BytesMut::new();

                buf.extend_from_slice(b"*");
                buf.extend_from_slice(frame_vec.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");

                for frame in frame_vec {
                    buf.extend_from_slice(&Box::pin(frame.serialize()).await?);
                }

                Ok(buf.freeze())
            }
            _ => unimplemented!(),
        }
    }

    /// associated function to deserialize a Frame from a bytes buffer
    /// bytes is a slice into the buffer, containing the whole frame to read
    /// should only be called if check returned Ok, indicating a complete frame
    /// bytes points to a static byte slice, so it's a reference to the whole frame buffer
    pub async fn deserialize(bytes: Bytes) -> Result<Frame> {
        // todo: implement deserialization
        match bytes[0] {
            b'+' => {
                // Simple string, slicing to ignore the leading + and ending CRLF char
                let bytes = &bytes[1..bytes.len() - 2];
                Ok(Frame::SimpleString(
                    String::from_utf8(bytes.to_vec()).unwrap(),
                ))
            }
            b'-' => {
                // Simple error, slicing to ignore the leading - and ending CRLF char
                let bytes = &bytes[1..bytes.len() - 2];
                Ok(Frame::SimpleError(
                    String::from_utf8(bytes.to_vec()).unwrap(),
                ))
            }
            b'$' => {
                // Bulk string, slicing to ignore the leading $ and ending CRLF char
                let bytes = &bytes[1..];
                let mut reader = bytes.reader();

                let mut buf_str1 = String::new();
                let mut buf_str2 = String::new();

                let _ = reader.read_line(&mut buf_str1).unwrap();
                let _ = reader.read_line(&mut buf_str2).unwrap();

                Ok(Frame::BulkString(
                    buf_str2.trim_end_matches("\r\n").to_string(),
                ))
            }
            _ => unimplemented!(),
        }
    }

    /// check whether the buffer contains a complete frame, starting from the current position
    /// note this function will consume the buffer
    pub async fn check(buf: &mut impl Buf) -> Result<()> {
        if buf.remaining() == 0 {
            return Err(wrap_error(RedisError::IncompleteFrame));
        }

        match buf.get_u8() {
            // simple string, simple error
            b'+' | b'-' => {
                let mut reader = buf.reader();

                let mut buf_str = String::new();

                let _ = reader.read_line(&mut buf_str).unwrap();

                if buf_str.ends_with("\r\n") {
                    Ok(())
                } else {
                    // fixme: there maybe edge cases here
                    Err(wrap_error(RedisError::IncompleteFrame))
                }
            }
            // bulk string
            b'$' => {
                let mut reader = buf.reader();

                let mut buf_str1 = String::new();
                let mut buf_str2 = String::new();

                let _ = reader.read_line(&mut buf_str1).unwrap();
                let _ = reader.read_line(&mut buf_str2).unwrap();

                // both lines should end with CRLF
                // an example RESP encodes bulk string:
                // $<length>\r\n<data>\r\n
                if !buf_str1.ends_with("\r\n") || !buf_str2.ends_with("\r\n") {
                    return Err(wrap_error(RedisError::IncompleteFrame));
                }

                let length = buf_str1.trim_end_matches("\r\n").parse::<usize>().unwrap();

                if length == buf_str2.len() - 2 {
                    Ok(())
                } else {
                    Err(wrap_error(RedisError::InvalidFrame))
                }
            }
            _ => Err(wrap_error(RedisError::InvalidFrame)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serialize_simple_string() {
        let frame = Frame::SimpleString("OK".to_string());
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"+OK\r\n"));
    }

    #[tokio::test]
    async fn test_serialize_simple_error() {
        let frame = Frame::SimpleError("ERR".to_string());
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"-ERR\r\n"));
    }

    #[tokio::test]
    async fn test_deserialize_simple_string() {
        let bytes = Bytes::from_static(b"+OK\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::SimpleString("OK".to_string()));
    }

    #[tokio::test]
    async fn test_deserialize_simple_error() {
        let bytes = Bytes::from_static(b"-ERR\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::SimpleError("ERR".to_string()));
    }

    #[tokio::test]
    async fn test_check_empty_buffer() {
        use std::io::Cursor;
        // a mutable buffer with the same underlying data to be shared across tests
        let buf = BytesMut::new();

        let mut buf_cursor = Cursor::new(&buf[..]);

        // empty buffer sould result in an error
        assert!(Frame::check(&mut buf_cursor).await.is_err());
    }

    #[tokio::test]
    async fn test_check_incomplete_frame() {
        use std::io::Cursor;
        // a mutable buffer with the same underlying data to be shared across tests
        let mut buf = BytesMut::new();

        buf.extend_from_slice(b"+OK");

        let mut buf_cursor = Cursor::new(&buf[..]);

        // an incomplete frame should result in an error
        assert!(Frame::check(&mut buf_cursor).await.is_err());
    }

    #[tokio::test]
    async fn test_check_complete_frame() {
        use std::io::Cursor;
        // a mutable buffer with the same underlying data to be shared across tests
        let mut buf = BytesMut::new();

        buf.extend_from_slice(b"+OK\r\n");

        let mut buf_cursor = Cursor::new(&buf[..]);

        // an incomplete frame should result in an error
        assert!(Frame::check(&mut buf_cursor).await.is_ok());
    }
}
