/// A Redis EXPIRE command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct Expire {
    key: String,
    seconds: i64,
}

impl Expire {
    /// Creates a new Expire command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set the expiration for
    /// * `seconds` - The number of seconds to set the expiration for
    ///
    /// # Returns
    ///
    /// A new Expire command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let expire = Expire::new("mykey", 60);
    /// ```
    pub fn new(key: &str, seconds: i64) -> Self {
        Self {
            key: key.to_string(),
            seconds,
        }
    }
}

impl Command for Expire {}

impl TryInto<Frame> for Expire {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("EXPIRE".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.seconds.to_string())))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expire() {
        let expire = Expire::new("mykey", 60);
        let frame: Frame = expire
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create EXPIRE command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("EXPIRE".into()),
                Frame::BulkString("mykey".into()),
                Frame::BulkString("60".into()),
            ])
        )
    }
}
