/// A Redis DECR command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
