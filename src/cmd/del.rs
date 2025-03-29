/// A Redis DEL command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
