/// A Redis INCR command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
