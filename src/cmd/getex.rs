/// A Redis GETEX command.
use crate::{Result, cmd::Command, frame::Frame};
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

impl Command for GetEx {}

impl TryInto<Frame> for GetEx {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("GETEX".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        if let Some(expiry) = self.expiry {
            match expiry {
                Expiry::EX(seconds) => {
                    frame.push_frame_to_array(Frame::BulkString("EX".into()))?;
                    frame.push_frame_to_array(Frame::Integer(seconds as i64))?;
                }
                Expiry::PX(milliseconds) => {
                    frame.push_frame_to_array(Frame::BulkString("PX".into()))?;
                    frame.push_frame_to_array(Frame::Integer(milliseconds as i64))?;
                }
                Expiry::EXAT(timestamp) => {
                    frame.push_frame_to_array(Frame::BulkString("EXAT".into()))?;
                    frame.push_frame_to_array(Frame::Integer(timestamp as i64))?;
                }
                Expiry::PXAT(timestamp) => {
                    frame.push_frame_to_array(Frame::BulkString("PXAT".into()))?;
                    frame.push_frame_to_array(Frame::Integer(timestamp as i64))?;
                }
                Expiry::PERSIST => {
                    frame.push_frame_to_array(Frame::BulkString("PERSIST".into()))?;
                }
            }
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let getex = GetEx::new("mykey", None);
        let frame: Frame = getex
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create GETEX command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("GETEX".into()),
                Frame::BulkString("mykey".into()),
            ])
        )
    }
}
