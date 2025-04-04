/// A Redis RPUSH command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct RPush {
    key: String,
    values: Vec<Vec<u8>>,
}

impl RPush {
    /// Creates a new RPUSH command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to push to
    /// * `values` - The values to push
    ///
    /// # Returns
    ///
    /// A new RPUSH command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let rpush = RPush::new("mylist", vec!["value1", "value2"]);
    /// ```
    pub fn new(key: &str, values: Vec<&[u8]>) -> Self {
        Self {
            key: key.to_string(),
            values: values.iter().map(|s| s.to_vec()).collect(),
        }
    }
}

impl Command for RPush {}

impl TryInto<Frame> for RPush {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("RPUSH".into()))?;
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
    fn test_rpush() {
        let rpush = RPush::new("mylist", vec!["value1".as_bytes(), "value2".as_bytes()]);
        let frame: Frame = rpush
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create RPUSH command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("RPUSH".into()),
                Frame::BulkString("mylist".into()),
                Frame::BulkString("value1".into()),
                Frame::BulkString("value2".into()),
            ])
        )
    }
}
