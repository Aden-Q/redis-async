/// A Redis EXISTS command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Exists {
    keys: Vec<String>,
}

impl Exists {
    /// Creates a new Exists command.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to check for existence in the Redis server
    ///
    /// # Returns
    ///
    /// A new Exists command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let exists = Exists::new(vec!["key1", "key2"]);
    /// ```
    pub fn new(keys: Vec<&str>) -> Self {
        Self {
            keys: keys.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Command for Exists {}

impl TryInto<Frame> for Exists {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("EXISTS".into()))?;

        for key in self.keys {
            frame.push_frame_to_array(Frame::BulkString(Bytes::from(key)))?;
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exists() {
        let exists = Exists::new(vec!["key1", "key2"]);
        let frame: Frame = exists
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create EXISTS command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("EXISTS".into()),
                Frame::BulkString("key1".into()),
                Frame::BulkString("key2".into()),
            ])
        )
    }
}
