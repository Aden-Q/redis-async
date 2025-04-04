/// A Redis TTL command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Ttl {
    key: String,
}

impl Ttl {
    /// Creates a new TTL command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get the expiration time for
    ///
    /// # Returns
    ///
    /// A new TTL command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ttl = Ttl::new("mykey");
    /// ```
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

impl Command for Ttl {}

impl TryInto<Frame> for Ttl {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("TTL".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl() {
        let ttl = Ttl::new("mykey");
        let frame: Frame = ttl
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create TTL command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("TTL".into()),
                Frame::BulkString("mykey".into()),
            ])
        );
    }
}
