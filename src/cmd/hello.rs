/// A Redis HELLO command.
use crate::{Result, cmd::Command, frame::Frame};

pub struct Hello {
    proto: Option<u8>,
}

impl Hello {
    /// Creates a new Hello command.
    ///
    /// # Arguments
    ///
    /// * `msg` - An optional message to send with ping
    ///
    /// # Returns
    ///
    /// A new Ping command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let hello = Hello::new(Some("hello".into()));
    /// ```
    pub fn new(proto: Option<u8>) -> Self {
        Self { proto }
    }
}

impl Command for Hello {}

impl TryInto<Frame> for Hello {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("HELLO".into()))?;

        // do not push the message if it is None
        if let Some(proto) = self.proto {
            frame.push_frame_to_array(Frame::BulkString(proto.to_string().into()))?;
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let hello = Hello::new(None);
        let frame: Frame = hello
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create HELLO command: {:?}", err));

        assert_eq!(frame, Frame::Array(vec![Frame::BulkString("HELLO".into())]));

        let hello = Hello::new(Some(3));
        let frame: Frame = hello
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create HELLO command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("HELLO".into()),
                Frame::BulkString(3.to_string().into()),
            ])
        );
    }
}
