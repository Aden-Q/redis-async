//! Redis commands.

use bytes::Bytes;

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
    pub fn new(msg: Option<&str>) -> Self {
        Self {
            msg: msg.map(|s| s.to_string()),
        }
    }
}

impl Command for Ping {
    /// Converts the ping command into a Frame to be transimitted over the stream.
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("PING".into()));

        // do not push the message if it is None
        if let Some(msg) = self.msg {
            frame.push_frame_to_array(Frame::BulkString(Bytes::from(msg)));
        }

        frame
    }
}

/// A Redis GET command.
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

impl Command for Get {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("GET".into()));
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)));

        frame
    }
}

/// A Redis SET command.
pub struct Set {
    key: String,
    value: String,
}

impl Set {
    /// Creates a new Set command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set in the Redis server
    /// * `value` - The value to set in the Redis server
    ///
    /// # Returns
    ///
    /// A new Set command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let set = Set::new("mykey", "myvalue");
    /// ```
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

impl Command for Set {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("SET".into()));
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)));
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.value)));

        frame
    }
}

/// A Redis DEL command.
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

impl Command for Del {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("DEL".into()));

        for key in self.keys {
            frame.push_frame_to_array(Frame::BulkString(Bytes::from(key)));
        }

        frame
    }
}

#[allow(dead_code)]
pub struct Publish {
    channel: String,
    message: String,
}

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

        assert_eq!(frame, Frame::Array(vec![Frame::BulkString("PING".into())]));

        let ping = Ping::new(Some("hello"));
        let frame = ping.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("PING".into()),
                Frame::BulkString("hello".into())
            ])
        );
    }

    #[test]
    fn test_get() {
        let get = Get::new("mykey");
        let frame = get.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("GET".into()),
                Frame::BulkString("mykey".into())
            ])
        );
    }

    #[test]
    fn test_set() {
        let set = Set::new("mykey", "myvalue");
        let frame = set.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("SET".into()),
                Frame::BulkString("mykey".into()),
                Frame::BulkString("myvalue".into()),
            ])
        )
    }

    #[test]
    fn test_del() {
        let del = Del::new(vec!["key1", "key2"]);
        let frame = del.into_stream();

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
