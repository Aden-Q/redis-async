/// A Redis LPOP command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct LPop {
    key: String,
    count: Option<u64>,
}

impl LPop {
    pub fn new(key: &str, count: Option<u64>) -> Self {
        Self {
            key: key.to_string(),
            count,
        }
    }
}

impl Command for LPop {}

impl TryInto<Frame> for LPop {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("LPOP".into()))?;
        frame.push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))?;

        if let Some(count) = self.count {
            frame.push_frame_to_array(Frame::Integer(count as i64))?;
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lpop() {
        let lpop = LPop::new("mylist", None);
        let frame: Frame = lpop
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create LPOP command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("LPOP".into()),
                Frame::BulkString("mylist".into())
            ])
        );

        let lpop = LPop::new("mylist", Some(2));
        let frame: Frame = lpop
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create LPOP command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("LPOP".into()),
                Frame::BulkString("mylist".into()),
                Frame::Integer(2)
            ])
        );
    }
}
