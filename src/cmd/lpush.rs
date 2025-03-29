/// A Redis LPUSH command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

pub struct LPush {
    key: String,
    values: Vec<String>,
}

impl LPush {
    /// Creates a new LPUSH command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to push to
    /// * `values` - The values to push
    ///
    /// # Returns
    ///
    /// A new LPUSH command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let lpush = LPush::new("mylist", vec!["value1", "value2"]);
    /// ```
    pub fn new(key: &str, values: Vec<&str>) -> Self {
        Self {
            key: key.to_string(),
            values: values.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Command for LPush {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("LPUSH".into()))
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
    fn test_lpush() {
        let lpush = LPush::new("mylist", vec!["value1", "value2"]);
        let frame = lpush.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("LPUSH".into()),
                Frame::BulkString("mylist".into()),
                Frame::BulkString("value1".into()),
                Frame::BulkString("value2".into()),
            ])
        )
    }
}
