/// A Redis PING command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

pub struct Ping {
    msg: Option<Bytes>,
}

impl Ping {
    /// Creates a new Ping command.
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
    /// let ping = Ping::new(Some("hello".into()));
    /// ```
    pub fn new(msg: Option<&[u8]>) -> Self {
        Self {
            msg: msg.map(|m| Bytes::from(m.to_vec())),
        }
    }
}

impl Command for Ping {
    /// Converts the ping command into a Frame to be transimitted over the stream.
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("PING".into()))
            .unwrap();

        // do not push the message if it is None
        if let Some(msg) = self.msg {
            frame.push_frame_to_array(Frame::BulkString(msg)).unwrap();
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping() {
        let ping = Ping::new(None);
        let frame = ping.into_stream();

        assert_eq!(frame, Frame::Array(vec![Frame::BulkString("PING".into())]));

        let ping = Ping::new(Some("hello".as_bytes()));
        let frame = ping.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("PING".into()),
                Frame::BulkString("hello".into())
            ])
        );
    }
}
