//! Implements the [RESP3](https://redis.io/docs/latest/develop/reference/protocol-spec)
//! serialization protocol for Redis client-server communication.

use crate::{RedisError, Result};
use bytes::{Buf, Bytes, BytesMut};
use std::io::{BufRead, Cursor};

#[derive(Debug, PartialEq)]
pub struct BigInt {
    sign: bool,
    data: Vec<u8>,
}

/// Frame represents a single RESP data transmit unit over the socket.
///
/// more on the RESP protocol can be found [here](https://redis.io/topics/protocol)
#[derive(Debug, PartialEq)]
pub enum Frame {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(Bytes),
    Array(Vec<Frame>),
    Null,
    Boolean(bool),
    Double(f64),
    BigNumber(BigInt),
    BulkError(Bytes),
    Map(Vec<(Frame, Frame)>),
    Attribute,
    Set(Vec<Frame>),
    Push,
}

impl Frame {
    /// Returns an empty Array Frame.
    pub const fn array() -> Self {
        Frame::Array(Vec::new())
    }

    /// A utility method to push a Frame into an Array Frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - A Frame to be pushed into the Array
    ///
    /// # Panics
    ///
    /// This method will panic if the Frame is not an Array
    pub fn push_frame_to_array(&mut self, frame: Frame) -> Result<()> {
        match self {
            Frame::Array(vec) => {
                vec.push(frame);
                Ok(())
            }
            _ => Err(RedisError::Unknown),
        }
    }

    /// Serializes a Frame into a bytes buffer.
    ///
    /// The returned value is a smart pointer only counting reference. It is cheap to clone.
    /// Caller can get the underlying slice by calling `as_slice` or `as_ref` on the returned value.
    /// It is almost 0 cost to get the slice.
    ///
    /// # Returns
    ///
    /// A Result containing the serialized bytes buffer
    pub async fn serialize(&self) -> Result<Bytes> {
        match self {
            Frame::SimpleString(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 3);

                // + indicates it is a simple string
                buf.extend_from_slice(b"+");
                // encode the string value
                buf.extend_from_slice(val.as_bytes());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::SimpleError(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 3);

                // - indicates it is an error
                buf.extend_from_slice(b"-");
                // encode the error message
                buf.extend_from_slice(val.as_bytes());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::Integer(val) => {
                let mut buf = BytesMut::with_capacity(20);

                // : indicates it is an integer
                buf.extend_from_slice(b":");
                // encode the integer value
                buf.extend_from_slice(val.to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::BulkString(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 5);

                // * indicates it is a bulk string
                buf.extend_from_slice(b"$");
                // encode the length of the binary string
                buf.extend_from_slice(val.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                // encode the binary string
                buf.extend_from_slice(val.as_ref());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::Array(frame_vec) => {
                let mut buf = BytesMut::new();

                // * indicates it is an array
                buf.extend_from_slice(b"*");
                // encode the number of elements in the array
                buf.extend_from_slice(frame_vec.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");

                // encode each element in the array
                for frame in frame_vec {
                    buf.extend_from_slice(&Box::pin(frame.serialize()).await?);
                }

                Ok(buf.freeze())
            }
            Frame::Null => {
                let mut buf = BytesMut::with_capacity(3);

                // _ indicates it is a null
                buf.extend_from_slice(b"_\r\n");

                Ok(buf.freeze())
            }
            Frame::Boolean(val) => {
                let mut buf: BytesMut = BytesMut::with_capacity(3);

                // # indicates it is a boolean
                buf.extend_from_slice(b"#");
                // encode the boolean value
                buf.extend_from_slice(if *val { b"t" } else { b"f" });
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::Double(val) => {
                todo!("Double serialization is not implemented yet {:?}", val)
            }
            Frame::BulkError(val) => {
                todo!("BulkError serialization is not implemented yet {:?}", val)
            }
            Frame::BigNumber(val) => {
                todo!("BigNumber serialization is not implemented yet {:?}", val)
            }
            Frame::Map(val) => {
                todo!("Map serialization is not implemented yet {:?}", val)
            }
            Frame::Attribute => {
                todo!("Attribute serialization is not implemented yet")
            }
            Frame::Set(val) => {
                todo!("Set serialization is not implemented yet {:?}", val)
            }
            Frame::Push => {
                todo!("Push serialization is not implemented yet")
            }
        }
    }

    /// Deserializes from the buffer into a Frame.
    ///
    /// The method reads from the buffer and parses it into a Frame.
    ///
    /// # Arguments
    ///
    /// * `buf` - An immutable read buffer containing the serialized Frame
    ///
    /// # Returns
    ///
    /// A Result containing the deserialized Frame
    pub async fn deserialize(buf: Bytes) -> Result<Frame> {
        // the cursor is almost zero cost as it is just a smart ptr to the buffer
        Frame::try_parse(&mut Cursor::new(&buf[..]))
    }

    /// Tries parsing a Frame from the buffer.
    ///
    /// This method wraps the input with a cursor to track the current version as we need to make resursive calls.
    /// Using a cursor avoids the need to split the buffer or passing an additional parameter.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` if the buffer contains a complete frame, the number of bytes needed to parse the frame
    /// * `Err(RedisError::IncompleteFrame)` if the buffer contains an incomplete frame
    /// * `Err(RedisError::InvalidFrame)` if the buffer contains an invalid frame
    pub fn try_parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame> {
        if !cursor.has_remaining() {
            return Err(RedisError::IncompleteFrame);
        }

        match cursor.get_u8() {
            b'+' => {
                // Simple string
                let mut buf = String::new();
                let _ = cursor.read_line(&mut buf).unwrap();

                if buf.ends_with("\r\n") {
                    Ok(Frame::SimpleString(
                        buf.trim_end_matches("\r\n").to_string(),
                    ))
                } else {
                    // fixme: there maybe edge cases here
                    // we need to guarantee there's no more \r\n in the buffer
                    Err(RedisError::IncompleteFrame)
                }
            }
            b'-' => {
                // Simple error
                let mut buf = String::new();
                let _ = cursor.read_line(&mut buf).unwrap();

                if buf.ends_with("\r\n") {
                    Ok(Frame::SimpleError(buf.trim_end_matches("\r\n").to_string()))
                } else {
                    // fixme: there maybe edge cases here
                    // we need to guarantee there's no more \r\n in the buffer
                    Err(RedisError::IncompleteFrame)
                }
            }
            b':' => {
                // Integer
                let mut buf = String::new();
                let _ = cursor.read_line(&mut buf).unwrap();

                // todo: check whether it is a valid integer
                if buf.ends_with("\r\n") {
                    Ok(Frame::Integer(
                        buf.trim_end_matches("\r\n").parse::<i64>().unwrap(),
                    ))
                } else {
                    Err(RedisError::IncompleteFrame)
                }
            }
            b'$' => {
                // Bulk string
                let mut buf = String::new();
                // read the length of the bulk string
                let _ = cursor.read_line(&mut buf).unwrap();

                if !buf.ends_with("\r\n") {
                    return Err(RedisError::IncompleteFrame);
                }

                let len = buf.trim_end_matches("\r\n").parse::<isize>().unwrap();

                // for RESP2, -1 indicates a null bulk string
                if len == -1 {
                    return Ok(Frame::Null);
                }

                buf.clear();
                let _ = cursor.read_line(&mut buf).unwrap();

                // -2 because \r\n
                if len as usize == buf.len() - 2 {
                    Ok(Frame::BulkString(Bytes::from(
                        buf.trim_end_matches("\r\n").to_string(),
                    )))
                } else {
                    Err(RedisError::InvalidFrame)
                }
            }
            b'*' => {
                // Array
                let mut buf = String::new();
                let _ = cursor.read_line(&mut buf).unwrap();

                let len = buf.trim_end_matches("\r\n").parse::<usize>().unwrap();

                let mut frame_vec: Vec<_> = Vec::with_capacity(len);

                for _ in 0..len {
                    frame_vec.push(Frame::try_parse(cursor)?);
                }

                Ok(Frame::Array(frame_vec))
            }
            b'_' => Ok(Frame::Null),
            b'#' => {
                // Boolean
                let mut buf = String::new();
                let _ = cursor.read_line(&mut buf).unwrap();

                if buf.ends_with("\r\n") {
                    let val = buf.trim_end_matches("\r\n");
                    if val == "t" {
                        Ok(Frame::Boolean(true))
                    } else if val == "f" {
                        Ok(Frame::Boolean(false))
                    } else {
                        Err(RedisError::InvalidFrame)
                    }
                } else {
                    Err(RedisError::IncompleteFrame)
                }
            }
            b',' => {
                // Double
                todo!("Double deserialization is not implemented yet")
            }
            b'(' => {
                // Big number
                todo!("Big number deserialization is not implemented yet")
            }
            b'!' => {
                // Bulk error
                todo!("Bulk error deserialization is not implemented yet")
            }
            b'%' => {
                // Map
                todo!("Map deserialization is not implemented yet")
            }
            b'&' => {
                // Attribute
                todo!("Attribute deserialization is not implemented yet")
            }
            b'~' => {
                // Set
                todo!("Set deserialization is not implemented yet")
            }
            b'>' => {
                // Push
                todo!("Push deserialization is not implemented yet")
            }
            _ => Err(RedisError::InvalidFrame),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the serialization of a simple string frame.
    #[tokio::test]
    async fn test_serialize_simple_string() {
        let frame = Frame::SimpleString("OK".to_string());
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"+OK\r\n"));
    }

    /// Tests the serialization of a simple error frame.
    #[tokio::test]
    async fn test_serialize_simple_error() {
        let frame = Frame::SimpleError("ERR".to_string());
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"-ERR\r\n"));
    }

    /// Tests the serialization of an integer frame.
    #[tokio::test]
    async fn test_serialize_integer() {
        // positive integer
        let frame = Frame::Integer(123_i64);
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b":123\r\n"));

        // negative integer
        let frame = Frame::Integer(-123_i64);
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b":-123\r\n"));
    }

    /// Tests the serialization of a bulk string frame.
    #[tokio::test]
    async fn test_serialize_bulk_string() {
        let frame = Frame::BulkString(Bytes::from_static(b"Hello Redis"));
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"$11\r\nHello Redis\r\n"));

        // empty bulk string
        let frame = Frame::BulkString(Bytes::from_static(b""));
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"$0\r\n\r\n"));
    }

    /// Tests the serailization of an array frame.
    #[tokio::test]
    async fn test_serialize_array() {
        let mut frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap();

        let bytes = frame.serialize().await.unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(b"*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n")
        );

        // empty array
        let frame = Frame::array();
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"*0\r\n"));

        // nested array
        let mut frame: Frame = Frame::array();
        let mut nested_frame = Frame::array();
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap();
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap();

        if let Frame::Array(vec) = &mut frame {
            vec.push(nested_frame);
        }

        let bytes = frame.serialize().await.unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(b"*1\r\n*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n")
        );
    }

    /// Tests the serialization of a null frame.
    #[tokio::test]
    async fn test_serialize_null() {
        let frame = Frame::Null;
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"_\r\n"));
    }

    /// Tests the serialization of a boolean frame.
    #[tokio::test]
    async fn test_serialize_boolean() {
        let frame = Frame::Boolean(true);
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"#t\r\n"));

        let frame = Frame::Boolean(false);
        let bytes = frame.serialize().await.unwrap();

        assert_eq!(bytes, Bytes::from_static(b"#f\r\n"));
    }

    /// Tests the deserialization of a simple string frame.
    #[tokio::test]
    async fn test_deserialize_simple_string() {
        let bytes = Bytes::from_static(b"+OK\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::SimpleString("OK".to_string()));
    }

    /// Tests the deserialization of a simple error frame.
    #[tokio::test]
    async fn test_deserialize_simple_error() {
        let bytes = Bytes::from_static(b"-ERR\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::SimpleError("ERR".to_string()));
    }

    /// Tests the deserialization of an integer frame.
    #[tokio::test]
    async fn test_deserialize_integer() {
        // positive integer
        let bytes = Bytes::from_static(b":123\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::Integer(123_i64));

        // negative integer
        let bytes = Bytes::from_static(b":-123\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::Integer(-123_i64));
    }

    /// Tests the deserialization of a bulk string frame.
    #[tokio::test]
    async fn test_deserialize_bulk_string() {
        let bytes = Bytes::from_static(b"$11\r\nHello Redis\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::BulkString(Bytes::from_static(b"Hello Redis")));

        let bytes = Bytes::from_static(b"$0\r\n\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::BulkString(Bytes::from_static(b"")));
    }

    /// Tests deseaialization of an array frame.
    #[tokio::test]
    async fn test_deserialize_array() {
        let bytes = Bytes::from_static(b"*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        let mut expected_frame = Frame::array();
        expected_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap();
        expected_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap();

        assert_eq!(frame, expected_frame);

        // empty array
        let bytes = Bytes::from_static(b"*0\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::array());

        // nested array
        let bytes = Bytes::from_static(b"*1\r\n*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        let mut expected_frame = Frame::array();
        let mut nested_frame = Frame::array();
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap();
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap();

        expected_frame.push_frame_to_array(nested_frame).unwrap();

        assert_eq!(frame, expected_frame);
    }

    /// Tests the deserialization of a null frame.
    #[tokio::test]
    async fn test_deserialize_null() {
        let bytes = Bytes::from_static(b"_\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::Null);
    }

    /// Tests the deserialization of a boolean frame.
    #[tokio::test]
    async fn test_deserialize_boolean() {
        let bytes = Bytes::from_static(b"#t\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::Boolean(true));

        let bytes = Bytes::from_static(b"#f\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap();

        assert_eq!(frame, Frame::Boolean(false));
    }
}
