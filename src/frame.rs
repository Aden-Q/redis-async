//! Implements the [RESP3](https://redis.io/docs/latest/develop/reference/protocol-spec)
//! serialization protocol for Redis client-server communication.

use crate::{RedisError, Result};
// use anyhow::Ok; // Removed as it conflicts with the Result type in your crate
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
    // first: encoding, second: data payload
    VerbatimString(Bytes, Bytes),
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

    /// A utility method to push a Frame into an Array/Set Frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - A Frame to be pushed into the Array
    ///
    /// # Panics
    ///
    /// This method will panic if the Frame is not an Array or Set.
    pub fn push_frame_to_array(&mut self, frame: Frame) -> Result<()> {
        match self {
            Frame::Array(vec) | Frame::Set(vec) => {
                vec.push(frame);
                Ok(())
            }
            _ => Err(RedisError::Unknown),
        }
    }

    /// A utility method to push a Frame into a Map Frame.
    ///
    /// # Arguments
    ///
    /// * `key` - A Frame to be used as a key in the Map
    /// * `value` - A Frame to be used as a value in the Map
    ///
    /// # Panics
    ///
    /// This method will panic if the Frame is not a Map.
    pub fn push_frame_to_map(&mut self, key: Frame, value: Frame) -> Result<()> {
        match self {
            Frame::Map(vec) => {
                vec.push((key, value));
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

                Ok(buf.freeze()) // Ensure this uses the crate's Result type
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

                // $ indicates it is a bulk string
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
                let mut buf: BytesMut = BytesMut::with_capacity(20);

                // , indicates it is a double
                buf.extend_from_slice(b",");

                // encode the double value
                if val.is_nan() {
                    buf.extend_from_slice(b"nan");
                } else {
                    match *val {
                        f64::INFINITY => buf.extend_from_slice(b"inf"),
                        f64::NEG_INFINITY => buf.extend_from_slice(b"-inf"),
                        _ => {
                            buf.extend_from_slice(val.to_string().as_bytes());
                        }
                    }
                }

                // append \r\n to the end of the buffer
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::BigNumber(val) => {
                todo!("BigNumber serialization is not implemented yet {:?}", val)
            }
            Frame::BulkError(val) => {
                let mut buf = BytesMut::with_capacity(val.len() + 5);

                // ! indicates it is a bulk error
                buf.extend_from_slice(b"!");
                // encode the length of the binary string
                buf.extend_from_slice(val.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                // encode the binary string
                buf.extend_from_slice(val.as_ref());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::VerbatimString(encoding, val) => {
                let mut buf: BytesMut = BytesMut::with_capacity(val.len() + 10);

                // = indicates it is a verbatim string
                buf.extend_from_slice(b"=");
                // encode the length of the binary string
                // +4 because encoding takes 3 bytes and : takes 1 byte
                buf.extend_from_slice((val.len() + 4).to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                // encode the encoding
                buf.extend_from_slice(encoding.as_ref());
                buf.extend_from_slice(b":");
                // encode the binary string
                buf.extend_from_slice(val.as_ref());
                buf.extend_from_slice(b"\r\n");

                Ok(buf.freeze())
            }
            Frame::Map(val) => {
                let mut buf: BytesMut = BytesMut::new();

                // % indicates it is a map
                buf.extend_from_slice(b"%");
                // encode the number of elements in the map
                buf.extend_from_slice(val.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");

                // encode each element in the map
                for (key, value) in val {
                    buf.extend_from_slice(&Box::pin(key.serialize()).await?);
                    buf.extend_from_slice(&Box::pin(value.serialize()).await?);
                }

                Ok(buf.freeze())
            }
            Frame::Attribute => {
                todo!("Attribute serialization is not implemented yet")
            }
            Frame::Set(val) => {
                let mut buf: BytesMut = BytesMut::new();

                // ~ indicates it is a set
                buf.extend_from_slice(b"~");
                // encode the number of elements in the set
                buf.extend_from_slice(val.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");

                // encode each element in the set
                for frame in val {
                    buf.extend_from_slice(&Box::pin(frame.serialize()).await?);
                }

                Ok(buf.freeze())
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
                cursor.read_line(&mut buf)?;

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
                cursor.read_line(&mut buf)?;

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
                cursor.read_line(&mut buf)?;

                // todo: check whether it is a valid integer
                if buf.ends_with("\r\n") {
                    Ok(Frame::Integer(buf.trim_end_matches("\r\n").parse::<i64>()?))
                } else {
                    Err(RedisError::IncompleteFrame)
                }
            }
            b'$' => {
                // Bulk string
                let mut buf = String::new();
                // read the length of the bulk string
                cursor.read_line(&mut buf)?;

                if !buf.ends_with("\r\n") {
                    return Err(RedisError::IncompleteFrame);
                }

                let len: isize = buf.trim_end_matches("\r\n").parse::<isize>()?;

                // for RESP2, -1 indicates a null bulk string
                if len == -1 {
                    return Ok(Frame::Null);
                }

                // +2 because \r\n
                if cursor.remaining() < len as usize + 2 {
                    return Err(RedisError::IncompleteFrame);
                }

                let data = Bytes::copy_from_slice(&cursor.chunk()[..len as usize]);

                // advance cursor
                cursor.advance(len as usize + 2);

                Ok(Frame::BulkString(data))
            }
            b'*' => {
                // Array
                let mut buf = String::new();
                cursor.read_line(&mut buf)?;

                let len = buf.trim_end_matches("\r\n").parse::<usize>()?;
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
                cursor.read_line(&mut buf)?;

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
                let mut buf = String::new();
                cursor.read_line(&mut buf)?;

                if buf.ends_with("\r\n") {
                    let val = buf.trim_end_matches("\r\n");
                    if val == "nan" {
                        Ok(Frame::Double(f64::NAN))
                    } else if val == "inf" {
                        Ok(Frame::Double(f64::INFINITY))
                    } else if val == "-inf" {
                        Ok(Frame::Double(f64::NEG_INFINITY))
                    } else {
                        Ok(Frame::Double(
                            val.parse::<f64>().map_err(|_| RedisError::InvalidFrame)?,
                        ))
                    }
                } else {
                    Err(RedisError::IncompleteFrame)
                }
            }
            b'(' => {
                // Big number
                todo!("Big number deserialization is not implemented yet")
            }
            b'!' => {
                // Bulk error
                let mut buf = String::new();
                // read the length of the bulk string
                cursor.read_line(&mut buf)?;

                if !buf.ends_with("\r\n") {
                    return Err(RedisError::IncompleteFrame);
                }

                let len: isize = buf.trim_end_matches("\r\n").parse::<isize>()?;

                // for RESP2, -1 indicates a null bulk error
                if len == -1 {
                    return Ok(Frame::Null);
                }

                let len: usize = len.try_into()?;

                // +2 because \r\n
                if cursor.remaining() < len + 2 {
                    return Err(RedisError::IncompleteFrame);
                }

                // check if cursor ends with \r\n
                if cursor.chunk()[len] != b'\r' || cursor.chunk()[len + 1] != b'\n' {
                    return Err(RedisError::InvalidFrame);
                }

                let data = Bytes::copy_from_slice(&cursor.chunk()[..len]);

                // advance cursor
                cursor.advance(len + 2);

                Ok(Frame::BulkError(data))
            }
            b'=' => {
                // Verbatim string
                let mut buf = String::new();
                // read the length of the bulk string
                cursor.read_line(&mut buf)?;

                if !buf.ends_with("\r\n") {
                    return Err(RedisError::IncompleteFrame);
                }

                let len: usize = buf.trim_end_matches("\r\n").parse::<usize>()?;

                // +2 for \r\n
                if cursor.remaining() < len + 2 {
                    return Err(RedisError::IncompleteFrame);
                }

                // check if cursor ends with \r\n
                if !cursor.chunk()[len..].starts_with(b"\r\n") {
                    return Err(RedisError::InvalidFrame);
                }

                // read the encoding
                let mut data = Bytes::copy_from_slice(&cursor.chunk()[..len]);

                // split data into encoding and value, : as the delimiter
                let encoding: Bytes = data.split_to(3);

                // data[0] is b':', ignore it
                data.advance(1);

                // advance cursor
                cursor.advance(len + 2);

                Ok(Frame::VerbatimString(encoding, data))
            }
            b'%' => {
                // Map
                let mut buf = String::new();
                cursor.read_line(&mut buf)?;

                let len = buf.trim_end_matches("\r\n").parse::<usize>()?;
                let mut frame_vec: Vec<_> = Vec::with_capacity(len);

                for _ in 0..len {
                    let key = Frame::try_parse(cursor)?;
                    let value = Frame::try_parse(cursor)?;
                    frame_vec.push((key, value));
                }

                Ok(Frame::Map(frame_vec))
            }
            b'&' => {
                // Attribute
                todo!("Attribute deserialization is not implemented yet")
            }
            b'~' => {
                // Set
                let mut buf = String::new();
                cursor.read_line(&mut buf)?;

                let len = buf.trim_end_matches("\r\n").parse::<usize>()?;
                let mut frame_vec: Vec<_> = Vec::with_capacity(len);

                for _ in 0..len {
                    frame_vec.push(Frame::try_parse(cursor)?);
                }

                Ok(Frame::Set(frame_vec))
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
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize simple string frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"+OK\r\n"));
    }

    /// Tests the serialization of a simple error frame.
    #[tokio::test]
    async fn test_serialize_simple_error() {
        let frame = Frame::SimpleError("ERR".to_string());
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize simple error frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"-ERR\r\n"));
    }

    /// Tests the serialization of an integer frame.
    #[tokio::test]
    async fn test_serialize_integer() {
        // positive integer
        let frame = Frame::Integer(123_i64);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize integer frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b":123\r\n"));

        // negative integer
        let frame = Frame::Integer(-123_i64);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize integer frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b":-123\r\n"));
    }

    /// Tests the serialization of a bulk string frame.
    #[tokio::test]
    async fn test_serialize_bulk_string() {
        let frame = Frame::BulkString(Bytes::from_static(b"Hello Redis"));
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize bulk string frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"$11\r\nHello Redis\r\n"));

        // empty bulk string
        let frame = Frame::BulkString(Bytes::from_static(b""));
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize empty bulk string frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"$0\r\n\r\n"));
    }

    /// Tests the serailization of an array frame.
    #[tokio::test]
    async fn test_serialize_array() {
        let mut frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap_or_else(|err| panic!("Failed to serialize array frame: {:?}", err));
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap_or_else(|err| panic!("Failed to serialize array frame: {:?}", err));

        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize array frame: {:?}", err));

        assert_eq!(
            bytes,
            Bytes::from_static(b"*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n")
        );

        // empty array
        let frame = Frame::array();
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize empty array frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"*0\r\n"));

        // nested array
        let mut frame: Frame = Frame::array();
        let mut nested_frame = Frame::array();
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap_or_else(|err| panic!("Failed to serialize nested array frame: {:?}", err));
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap_or_else(|err| panic!("Failed to serialize nested array frame: {:?}", err));

        if let Frame::Array(vec) = &mut frame {
            vec.push(nested_frame);
        }

        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize nested array frame: {:?}", err));

        assert_eq!(
            bytes,
            Bytes::from_static(b"*1\r\n*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n")
        );
    }

    /// Tests the serialization of a null frame.
    #[tokio::test]
    async fn test_serialize_null() {
        let frame = Frame::Null;
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize null frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"_\r\n"));
    }

    /// Tests the serialization of a boolean frame.
    #[tokio::test]
    async fn test_serialize_boolean() {
        let frame = Frame::Boolean(true);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize boolean frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"#t\r\n"));

        let frame = Frame::Boolean(false);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize boolean frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"#f\r\n"));
    }

    // Tests the serialization of a double frame.
    #[tokio::test]
    async fn test_serialize_double() {
        let frame = Frame::Double(123.456);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize double frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b",123.456\r\n"));

        let frame = Frame::Double(f64::NAN);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize NaN frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b",nan\r\n"));

        let frame = Frame::Double(f64::INFINITY);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize infinity frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b",inf\r\n"));

        let frame = Frame::Double(f64::NEG_INFINITY);
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize negative infinity frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b",-inf\r\n"));
    }

    /// Tests the serialization of a bulk error frame.
    #[tokio::test]
    async fn test_serialize_bulk_error() {
        let frame = Frame::BulkError(Bytes::from_static(b"Hello Redis"));
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize bulk error frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"!11\r\nHello Redis\r\n"));

        // empty bulk error
        let frame = Frame::BulkError(Bytes::from_static(b""));
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize empty bulk error frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"!0\r\n\r\n"));
    }

    /// Tests the serialization of a verbatim string frame.
    #[tokio::test]
    async fn test_serialize_verbatim_string() {
        let frame = Frame::VerbatimString(
            Bytes::from_static(b"txt"),
            Bytes::from_static(b"Some string"),
        );
        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize verbatim string frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"=15\r\ntxt:Some string\r\n"));

        // empty verbatim string
        let frame = Frame::VerbatimString(Bytes::from_static(b"txt"), Bytes::from_static(b""));
        let bytes = frame.serialize().await.unwrap_or_else(|err| {
            panic!("Failed to serialize empty verbatim string frame: {:?}", err)
        });

        assert_eq!(bytes, Bytes::from_static(b"=4\r\ntxt:\r\n"));
    }

    /// Tests the serialization of a map frame.
    #[tokio::test]
    async fn test_serialize_map() {
        let mut frame: Frame = Frame::Map(Vec::new());
        frame
            .push_frame_to_map(
                Frame::SimpleString("key".to_string()),
                Frame::SimpleString("value".to_string()),
            )
            .unwrap_or_else(|err| panic!("Failed to serialize map frame: {:?}", err));

        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize map frame: {:?}", err));

        assert_eq!(bytes, Bytes::from_static(b"%1\r\n+key\r\n+value\r\n"));
    }

    /// Tests the serialization of a set frame.
    #[tokio::test]
    async fn test_serialize_set() {
        let mut frame: Frame = Frame::Set(Vec::new());
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap_or_else(|err| panic!("Failed to serialize set frame: {:?}", err));
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap_or_else(|err| panic!("Failed to serialize set frame: {:?}", err));

        let bytes = frame
            .serialize()
            .await
            .unwrap_or_else(|err| panic!("Failed to serialize set frame: {:?}", err));

        assert_eq!(
            bytes,
            Bytes::from_static(b"~2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n")
        );
    }

    /// Tests the deserialization of a simple string frame.
    #[tokio::test]
    async fn test_deserialize_simple_string() {
        let bytes = Bytes::from_static(b"+OK\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize simple string frame: {:?}", err));

        assert_eq!(frame, Frame::SimpleString("OK".to_string()));
    }

    /// Tests the deserialization of a simple error frame.
    #[tokio::test]
    async fn test_deserialize_simple_error() {
        let bytes = Bytes::from_static(b"-ERR\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize simple error frame: {:?}", err));

        assert_eq!(frame, Frame::SimpleError("ERR".to_string()));
    }

    /// Tests the deserialization of an integer frame.
    #[tokio::test]
    async fn test_deserialize_integer() {
        // positive integer
        let bytes = Bytes::from_static(b":123\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize integer frame: {:?}", err));

        assert_eq!(frame, Frame::Integer(123_i64));

        // negative integer
        let bytes = Bytes::from_static(b":-123\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap_or_else(|err| {
            panic!("Failed to deserialize negative integer frame: {:?}", err)
        });

        assert_eq!(frame, Frame::Integer(-123_i64));
    }

    /// Tests the deserialization of a bulk string frame.
    #[tokio::test]
    async fn test_deserialize_bulk_string() {
        let bytes = Bytes::from_static(b"$11\r\nHello Redis\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize bulk string frame: {:?}", err));

        assert_eq!(frame, Frame::BulkString(Bytes::from_static(b"Hello Redis")));

        let bytes = Bytes::from_static(b"$0\r\n\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap_or_else(|err| {
            panic!("Failed to deserialize empty bulk string frame: {:?}", err)
        });

        assert_eq!(frame, Frame::BulkString(Bytes::from_static(b"")));
    }

    /// Tests deseaialization of an array frame.
    #[tokio::test]
    async fn test_deserialize_array() {
        let bytes = Bytes::from_static(b"*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize array frame: {:?}", err));

        let mut expected_frame = Frame::array();
        expected_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap_or_else(|err| panic!("Failed to deserialize array frame: {:?}", err));
        expected_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap_or_else(|err| panic!("Failed to deserialize array frame: {:?}", err));

        assert_eq!(frame, expected_frame);

        // empty array
        let bytes = Bytes::from_static(b"*0\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize empty array frame: {:?}", err));

        assert_eq!(frame, Frame::array());

        // nested array
        let bytes = Bytes::from_static(b"*1\r\n*2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize nested array frame: {:?}", err));

        let mut expected_frame = Frame::array();
        let mut nested_frame = Frame::array();
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap_or_else(|err| panic!("Failed to deserialize nested array frame: {:?}", err));
        nested_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap_or_else(|err| panic!("Failed to deserialize nested array frame: {:?}", err));

        expected_frame
            .push_frame_to_array(nested_frame)
            .unwrap_or_else(|err| panic!("Failed to deserialize nested array frame: {:?}", err));

        assert_eq!(frame, expected_frame);
    }

    /// Tests the deserialization of a null frame.
    #[tokio::test]
    async fn test_deserialize_null() {
        let bytes = Bytes::from_static(b"_\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize null frame: {:?}", err));

        assert_eq!(frame, Frame::Null);
    }

    /// Tests the deserialization of a boolean frame.
    #[tokio::test]
    async fn test_deserialize_boolean() {
        let bytes = Bytes::from_static(b"#t\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize boolean frame: {:?}", err));

        assert_eq!(frame, Frame::Boolean(true));

        let bytes = Bytes::from_static(b"#f\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize false boolean frame: {:?}", err));

        assert_eq!(frame, Frame::Boolean(false));
    }

    /// Tests the deserialization of a double frame.
    #[tokio::test]
    async fn test_deserialize_double() {
        let bytes = Bytes::from_static(b",123.456\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize double frame: {:?}", err));

        assert_eq!(frame, Frame::Double(123.456));

        let bytes = Bytes::from_static(b",nan\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize NaN double frame: {:?}", err));

        if let Frame::Double(val) = frame {
            assert!(val.is_nan());
        } else {
            panic!("Expected a Double frame");
        }

        let bytes = Bytes::from_static(b",inf\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize infinity double frame: {:?}", err));

        assert_eq!(frame, Frame::Double(f64::INFINITY));

        let bytes = Bytes::from_static(b",-inf\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap_or_else(|err| {
            panic!(
                "Failed to deserialize negative infinity double frame: {:?}",
                err
            )
        });

        assert_eq!(frame, Frame::Double(f64::NEG_INFINITY));
    }

    /// Tests the deserialization of a bulk error frame.
    #[tokio::test]
    async fn test_deserialize_bulk_error() {
        let bytes = Bytes::from_static(b"!11\r\nHello Redis\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize bulk error frame: {:?}", err));

        assert_eq!(frame, Frame::BulkError(Bytes::from_static(b"Hello Redis")));

        let bytes = Bytes::from_static(b"!0\r\n\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap_or_else(|err| {
            panic!("Failed to deserialize empty bulk error frame: {:?}", err)
        });

        assert_eq!(frame, Frame::BulkError(Bytes::from_static(b"")));
    }

    /// Tests the deserialization of a verbatim string frame.
    #[tokio::test]
    async fn test_deserialize_verbatim_string() {
        let bytes = Bytes::from_static(b"=15\r\ntxt:Some string\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize verbatim string frame: {:?}", err));

        assert_eq!(
            frame,
            Frame::VerbatimString(
                Bytes::from_static(b"txt"),
                Bytes::from_static(b"Some string")
            )
        );

        let bytes = Bytes::from_static(b"=4\r\ntxt:\r\n");

        let frame = Frame::deserialize(bytes).await.unwrap_or_else(|err| {
            panic!(
                "Failed to deserialize empty verbatim string frame: {:?}",
                err
            )
        });

        assert_eq!(
            frame,
            Frame::VerbatimString(Bytes::from_static(b"txt"), Bytes::from_static(b""))
        );
    }

    /// Tests the deserialization of a map frame.
    #[tokio::test]
    async fn test_deserialize_map() {
        let bytes = Bytes::from_static(b"%1\r\n+key\r\n+value\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize map frame: {:?}", err));

        let mut expected_frame = Frame::Map(Vec::new());
        expected_frame
            .push_frame_to_map(
                Frame::SimpleString("key".to_string()),
                Frame::SimpleString("value".to_string()),
            )
            .unwrap_or_else(|err| panic!("Failed to deserialize map frame: {:?}", err));

        assert_eq!(frame, expected_frame);
    }

    /// Tests the deserialization of a set frame.
    #[tokio::test]
    async fn test_deserialize_set() {
        let bytes = Bytes::from_static(b"~2\r\n$5\r\nHello\r\n$5\r\nRedis\r\n");

        let frame = Frame::deserialize(bytes)
            .await
            .unwrap_or_else(|err| panic!("Failed to deserialize set frame: {:?}", err));

        let mut expected_frame = Frame::Set(Vec::new());
        expected_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Hello")))
            .unwrap_or_else(|err| panic!("Failed to deserialize set frame: {:?}", err));
        expected_frame
            .push_frame_to_array(Frame::BulkString(Bytes::from_static(b"Redis")))
            .unwrap_or_else(|err| panic!("Failed to deserialize set frame: {:?}", err));

        assert_eq!(frame, expected_frame);
    }
}
