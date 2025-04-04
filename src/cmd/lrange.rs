/// A Redis LRANGE command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct LRange {
    key: String,
    start: i64,
    end: i64,
}

impl LRange {
    pub fn new(key: &str, start: i64, end: i64) -> Self {
        Self {
            key: key.to_string(),
            start,
            end,
        }
    }
}

impl Command for LRange {}

impl TryInto<Frame> for LRange {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("LRANGE".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;
        frame.push_frame_to_array(Frame::Integer(self.start))?;
        frame.push_frame_to_array(Frame::Integer(self.end))?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lrange() {
        let lrange = LRange::new("mylist", 0, -1);
        let frame: Frame = lrange
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create LRANGE command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("LRANGE".into()),
                Frame::BulkString("mylist".into()),
                Frame::Integer(0),
                Frame::Integer(-1)
            ])
        );
    }
}
