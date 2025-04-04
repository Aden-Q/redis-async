/// A Redis DECR command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Decr {
    key: String,
}

impl Decr {
    /// Creates a new DECR command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to decrement
    ///
    /// # Returns
    ///
    /// A new DECR command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let decr = Decr::new("mykey");
    /// ```
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

impl Command for Decr {}

impl TryInto<Frame> for Decr {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("DECR".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decr() {
        let decr = Decr::new("mykey");
        let frame: Frame = decr
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create DECR command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("DECR".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }
}
