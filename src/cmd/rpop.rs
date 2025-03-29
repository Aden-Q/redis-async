/// A Redis RPOP command.
use crate::cmd::Command;
use crate::frame::Frame;
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

impl Command for RPop {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("RPOP".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();
        if let Some(count) = self.count {
            frame
                .push_frame_to_array(Frame::Integer(count as i64))
                .unwrap();
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpop() {
        let rpop = RPop::new("mylist", None);
        let frame = rpop.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("RPOP".into()),
                Frame::BulkString("mylist".into())
            ])
        );

        let rpop = RPop::new("mylist", Some(2));
        let frame = rpop.into_stream();

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
