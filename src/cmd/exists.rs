/// A Redis EXISTS command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
