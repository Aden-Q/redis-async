/// A Redis GET command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Get {
    key: String,
}

impl Get {
    /// Creates a new Get command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get from the Redis server
    ///
    /// # Returns
    ///
    /// A new Get command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let get = Get::new("mykey");
    /// ```
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

impl Command for Get {}

impl TryInto<Frame> for Get {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("GET".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let get = Get::new("mykey");
        let frame: Frame = get
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create GET command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("GET".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }
}
