/// A Redis RPUSH command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

pub struct RPush {
    key: String,
    values: Vec<String>,
}

impl RPush {
    /// Creates a new RPUSH command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to push to
    /// * `values` - The values to push
    ///
    /// # Returns
    ///
    /// A new RPUSH command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let rpush = RPush::new("mylist", vec!["value1", "value2"]);
    /// ```
    pub fn new(key: &str, values: Vec<&str>) -> Self {
        Self {
            key: key.to_string(),
            values: values.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Command for RPush {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("RPUSH".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();

        for value in self.values {
            frame
                .push_frame_to_array(Frame::BulkString(Bytes::from(value)))
                .unwrap();
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpush() {
        let rpush = RPush::new("mylist", vec!["value1", "value2"]);
        let frame = rpush.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("RPUSH".into()),
                Frame::BulkString("mylist".into()),
                Frame::BulkString("value1".into()),
                Frame::BulkString("value2".into()),
            ])
        )
    }
}
