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
        frame
            .push_frame_to_array(Frame::BulkString("PING".into()))
            .unwrap();

        // do not push the message if it is None
        if let Some(msg) = self.msg {
            frame
                .push_frame_to_array(Frame::BulkString(Bytes::from(msg)))
                .unwrap();
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
        frame
            .push_frame_to_array(Frame::BulkString("GET".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();

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
        frame
            .push_frame_to_array(Frame::BulkString("SET".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.value)))
            .unwrap();

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
        frame
            .push_frame_to_array(Frame::BulkString("DEL".into()))
            .unwrap();

        for key in self.keys {
            frame
                .push_frame_to_array(Frame::BulkString(Bytes::from(key)))
                .unwrap();
        }

        frame
    }
}

/// A Redis EXISTS command.
pub struct Exists {
    keys: Vec<String>,
}

impl Exists {
    /// Creates a new Exists command.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to check for existence in the Redis server
    ///
    /// # Returns
    ///
    /// A new Exists command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let exists = Exists::new(vec!["key1", "key2"]);
    /// ```
    pub fn new(keys: Vec<&str>) -> Self {
        Self {
            keys: keys.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Command for Exists {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("EXISTS".into()))
            .unwrap();

        for key in self.keys {
            frame
                .push_frame_to_array(Frame::BulkString(Bytes::from(key)))
                .unwrap();
        }

        frame
    }
}

/// A Redis EXPIRE command.
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

impl Command for Expire {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("EXPIRE".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.seconds.to_string())))
            .unwrap();

        frame
    }
}

/// A Redis TTL command.
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

impl Command for Ttl {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("TTL".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();

        frame
    }
}

/// A Redis INCR command.
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

impl Command for Incr {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("INCR".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();

        frame
    }
}

/// A Redis DECR command.
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

impl Command for Decr {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("DECR".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();

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

    #[test]
    fn test_exists() {
        let exists = Exists::new(vec!["key1", "key2"]);
        let frame = exists.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("EXISTS".into()),
                Frame::BulkString("key1".into()),
                Frame::BulkString("key2".into()),
            ])
        )
    }

    #[test]
    fn test_expire() {
        let expire = Expire::new("mykey", 60);
        let frame = expire.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("EXPIRE".into()),
                Frame::BulkString("mykey".into()),
                Frame::BulkString("60".into()),
            ])
        )
    }

    #[test]
    fn test_ttl() {
        let ttl = Ttl::new("mykey");
        let frame = ttl.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("TTL".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }

    #[test]
    fn test_incr() {
        let incr = Incr::new("mykey");
        let frame = incr.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("INCR".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }

    #[test]
    fn test_decr() {
        let decr = Decr::new("mykey");
        let frame = decr.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("DECR".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }
}
