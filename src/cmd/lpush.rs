/// A Redis LPUSH command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct LPush {
    key: String,
    values: Vec<Vec<u8>>,
}

impl LPush {
    /// Creates a new LPUSH command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to push to
    /// * `values` - The values to push
    ///
    /// # Returns
    ///
    /// A new LPUSH command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let lpush = LPush::new("mylist", vec!["value1", "value2"]);
    /// ```
    pub fn new(key: &str, values: Vec<&[u8]>) -> Self {
        Self {
            key: key.to_string(),
            values: values.iter().map(|s| s.to_vec()).collect(),
        }
    }
}

impl Command for LPush {}

impl TryInto<Frame> for LPush {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("LPUSH".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        for value in self.values {
            frame.push_frame_to_array(Frame::BulkString(Bytes::from(value)))?;
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lpush() {
        let lpush = LPush::new("mylist", vec![b"value1", b"value2"]);
        let frame: Frame = lpush
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create LPUSH command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("LPUSH".into()),
                Frame::BulkString("mylist".into()),
                Frame::BulkString("value1".into()),
                Frame::BulkString("value2".into()),
            ])
        )
    }
}
