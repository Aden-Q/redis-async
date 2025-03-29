/// A Redis LPOP command.
use crate::cmd::Command;
use crate::frame::Frame;
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

impl Command for LPop {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("LPOP".into()))
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
    fn test_lpop() {
        let lpop = LPop::new("mylist", None);
        let frame = lpop.into_stream();

        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString("LPOP".into()),
                Frame::BulkString("mylist".into())
            ])
        );

        let lpop = LPop::new("mylist", Some(2));
        let frame = lpop.into_stream();

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
