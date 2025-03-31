/// A Redis HELLO command.
use crate::cmd::Command;
use crate::frame::Frame;

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

impl Command for Hello {
    /// Converts the Hello command into a Frame to be transimitted over the stream.
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("HELLO".into()))
            .unwrap();

        // do not push the message if it is None
        if let Some(proto) = self.proto {
            frame
                .push_frame_to_array(Frame::Integer(proto as i64))
                .unwrap();
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let hello = Hello::new(None);
        let frame = hello.into_stream();

        assert_eq!(frame, Frame::Array(vec![Frame::BulkString("HELLO".into())]));

        let ping = Hello::new(Some(3));
        let frame = ping.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![Frame::BulkString("HELLO".into()), Frame::Integer(3),])
        );
    }
}
