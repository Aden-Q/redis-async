/// A Redis TTL command.
use crate::cmd::Command;
use crate::frame::Frame;
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
