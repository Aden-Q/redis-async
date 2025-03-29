/// A Redis SET command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
