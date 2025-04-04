/// A Redis DEL command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Del {
    keys: Vec<String>,
}

impl Del {
    /// Creates a new Del command.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to delete from the Redis server
    ///
    /// # Returns
    ///
    /// A new Del command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let del = Del::new(vec!["key1", "key2"]);
    /// ```
    pub fn new(keys: Vec<&str>) -> Self {
        Self {
            keys: keys.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Command for Del {}

impl TryInto<Frame> for Del {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("DEL".into()))?;

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
    fn test_del() {
        let del = Del::new(vec!["key1", "key2"]);
        let frame: Frame = del
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create DEL command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("DEL".into()),
                Frame::BulkString("key1".into()),
                Frame::BulkString("key2".into()),
            ])
        )
    }
}
