/// A Redis GETEX command.
use crate::cmd::Command;
use crate::frame::Frame;
use bytes::Bytes;

#[derive(Debug)]
pub enum Expiry {
    EX(u64),
    PX(u64),
    EXAT(u64),
    PXAT(u64),
    PERSIST,
}

#[derive(Debug)]
pub struct GetEx {
    key: String,
    expiry: Option<Expiry>,
}

impl GetEx {
    /// Creates a new GetEx command.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get from the Redis server
    /// * `expiry` - The expiry time for the key
    ///
    /// # Returns
    ///
    /// A new GetEx command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let getex = GetEx::new("mykey");
    /// ```
    pub fn new(key: &str, expiry: Option<Expiry>) -> Self {
        Self {
            key: key.to_string(),
            expiry,
        }
    }
}

impl Command for GetEx {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("GETEX".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();

        if let Some(expiry) = self.expiry {
            match expiry {
                Expiry::EX(seconds) => {
                    frame
                        .push_frame_to_array(Frame::BulkString("EX".into()))
                        .unwrap();
                    frame
                        .push_frame_to_array(Frame::Integer(seconds as i64))
                        .unwrap();
                }
                Expiry::PX(milliseconds) => {
                    frame
                        .push_frame_to_array(Frame::BulkString("PX".into()))
                        .unwrap();
                    frame
                        .push_frame_to_array(Frame::Integer(milliseconds as i64))
                        .unwrap();
                }
                Expiry::EXAT(timestamp) => {
                    frame
                        .push_frame_to_array(Frame::BulkString("EXAT".into()))
                        .unwrap();
                    frame
                        .push_frame_to_array(Frame::Integer(timestamp as i64))
                        .unwrap();
                }
                Expiry::PXAT(timestamp) => {
                    frame
                        .push_frame_to_array(Frame::BulkString("PXAT".into()))
                        .unwrap();
                    frame
                        .push_frame_to_array(Frame::Integer(timestamp as i64))
                        .unwrap();
                }
                Expiry::PERSIST => {
                    frame
                        .push_frame_to_array(Frame::BulkString("PERSIST".into()))
                        .unwrap();
                }
            }
        }
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let getex = GetEx::new("mykey", None);
        let frame = getex.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("GETEX".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }
}
