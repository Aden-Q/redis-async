/// A Redis SET command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

/// A Redis SET command.
pub struct Set {
    key: String,
    value: Bytes,
    _options: Option<Vec<String>>,
}

impl Set {
    /// Creates a new Set command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set in the Redis server
    /// * `value` - The value to set in the Redis server
    ///
    /// # Returns
    ///
    /// A new Set command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let set = Set::new("mykey", "myvalue");
    /// ```
    pub fn new(key: &str, value: &[u8]) -> Self {
        Self {
            key: key.to_string(),
            value: Bytes::copy_from_slice(value),
            _options: None,
        }
    }
}

impl Command for Set {}

impl TryInto<Frame> for Set {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("SET".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;
        frame.push_frame_to_array(Frame::BulkString(self.value))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let set = Set::new("mykey", "myvalue".as_bytes());
        let frame: Frame = set
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create SET command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("SET".into()),
                Frame::BulkString("mykey".into()),
                Frame::BulkString("myvalue".into()),
            ])
        )
    }
}
