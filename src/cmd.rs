//! Redis commands.

use crate::Frame;

/// A trait for all Redis commands.
pub trait Command {
    /// Converts the command into a Frame to be transimitted over the stream.
    fn into_stream(self) -> Frame;
}

/// A Redis PING command.
pub struct Ping {
    msg: Option<String>,
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
    pub fn new(msg: Option<String>) -> Self {
        Self { msg }
    }
}

impl Command for Ping {
    /// Converts the ping command into a Frame to be transimitted over the stream.
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame.push_bulk_str("ping".into());

        if let Some(msg) = self.msg {
            frame.push_bulk_str(msg);
        }

        frame
    }
}

#[allow(dead_code)]
pub struct Get {
    key: String,
}

impl Get {}

#[allow(dead_code)]
pub struct Publish {
    channel: String,
    message: String,
}

#[allow(dead_code)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {}

impl Publish {}

#[allow(dead_code)]
pub struct Subscribe {
    channels: Vec<String>,
}

impl Subscribe {}

#[allow(dead_code)]
pub struct Unsubscribe {
    channels: Vec<String>,
}

impl Unsubscribe {}

#[allow(dead_code)]
pub struct Unknown {
    command: String,
}

impl Unknown {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping() {
        let ping = Ping::new(None);
        let frame = ping.into_stream();

        assert_eq!(frame, Frame::Array(vec![Frame::BulkString("ping".into())]));

        let ping = Ping::new(Some("hello".into()));
        let frame = ping.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("ping".into()),
                Frame::BulkString("hello".into())
            ])
        );
    }
}
