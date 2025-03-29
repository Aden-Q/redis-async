/// A Redis LRANGE command.
use crate::cmd::Command;
use crate::frame::Frame;
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

impl Command for LRange {
    fn into_stream(self) -> Frame {
        let mut frame: Frame = Frame::array();
        frame
            .push_frame_to_array(Frame::BulkString("LRANGE".into()))
            .unwrap();
        frame
            .push_frame_to_array(Frame::BulkString(Bytes::from(self.key)))
            .unwrap();
        frame
            .push_frame_to_array(Frame::Integer(self.start))
            .unwrap();
        frame.push_frame_to_array(Frame::Integer(self.end)).unwrap();

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lrange() {
        let lrange = LRange::new("mylist", 0, -1);
        let frame = lrange.into_stream();

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
