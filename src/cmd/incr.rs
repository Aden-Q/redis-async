/// A Redis INCR command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Incr {
    key: String,
}

impl Incr {
    /// Creates a new INCR command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to increment
    ///
    /// # Returns
    ///
    /// A new INCR command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let incr = Incr::new("mykey");
    /// ```
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

impl Command for Incr {}

impl TryInto<Frame> for Incr {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("INCR".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incr() {
        let incr = Incr::new("mykey");
        let frame: Frame = incr
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create INCR command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("INCR".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }
}
