/// A Redis EXPIRE command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
