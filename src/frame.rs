// implements Redis serialization protocol (RESP)
// for client-server communication

use std::io::BufRead;

use crate::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};

// derive std error
// #[derive(Debug)]
// pub enum Error {
//     Incomplete,
//     Other(crate::Error),
// }

// Frame represents a single RESP frame
#[derive(Debug)]
pub enum Frame {
    // RESP data types
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(Bytes),
    Array(Vec<Frame>),
    Null,
    Boolean(bool),
}

// we need to implemenet a custom serialization protocol
// for Frame type
impl Frame {
    pub fn array() -> Self {
        Frame::Array(Vec::new())
    }

    pub fn push_bulk(&mut self, item: Bytes) {
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
            Frame::Array(val) => {
                let mut buf = BytesMut::new();

                // fix it: implement array serialization
                buf.extend_from_slice(b"*1\r\n$4\r\nping\r\n");

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
            b'+' | b'-' => {
                // Simple string, slicing to ignore the leading + and ending CRLF char
                let bytes = &bytes[1..bytes.len() - 2];
                Ok(Frame::SimpleString(
                    String::from_utf8(bytes.to_vec()).unwrap(),
                ))
            }
            _ => unimplemented!(),
        }
    }

    /// check whether the buffer contains a complete frame, starting from the current position
    /// note this function will consume the buffer
    pub fn check(buf: &mut impl Buf) -> Result<()> {
        if buf.remaining() == 0 {
            return Err("incomplete".into());
        }

        match buf.get_u8() {
            b'+' | b'-' => {
                let mut reader = buf.reader();

                let mut buf_str = String::new();

                let _ = reader.read_line(&mut buf_str).unwrap();

                if buf_str.ends_with("\r\n") {
                    Ok(())
                } else {
                    Err("incomplete".into())
                }
            }
            // create a custom error type
            _ => Err("protocol error".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serialize_simple() {
        let frame = Frame::SimpleString("OK".to_string());
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"+OK\r\n"));
    }

    #[tokio::test]
    async fn test_serialize_error() {
        let frame = Frame::SimpleString("ERR".to_string());
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"-ERR\r\n"));
    }

    #[test]
    fn test_check() {
        use std::io::Cursor;
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");

        let mut buf = Cursor::new(&buf[..]);

        Frame::check(&mut buf).unwrap();
    }
}
