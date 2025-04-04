/// A Redis RPOP command.
use crate::{Result, cmd::Command, frame::Frame};
use bytes::Bytes;

pub struct RPop {
    key: String,
    count: Option<u64>,
}

impl RPop {
    pub fn new(key: &str, count: Option<u64>) -> Self {
        Self {
            key: key.to_string(),
            count,
        }
    }
}

impl Command for RPop {}

impl TryInto<Frame> for RPop {
    type Error = crate::RedisError;

    fn try_into(self) -> Result<Frame> {
        let mut frame: Frame = Frame::array();
        frame.push_frame_to_array(Frame::BulkString("RPOP".into()))?;
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
    fn test_rpop() {
        let rpop = RPop::new("mylist", None);
        let frame: Frame = rpop
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create RPOP command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("RPOP".into()),
                Frame::BulkString("mylist".into())
            ])
        );

        let rpop = RPop::new("mylist", Some(2));
        let frame: Frame = rpop
            .try_into()
            .unwrap_or_else(|err| panic!("Failed to create RPOP command: {:?}", err));

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("RPOP".into()),
                Frame::BulkString("mylist".into()),
                Frame::Integer(2)
            ])
        );
    }
}
