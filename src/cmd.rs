use crate::Frame;
use bytes::Bytes;

pub trait Command {
    fn into_stream(self) -> Frame;
}

pub struct Ping {
    msg: String,
}

impl Ping {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl Command for Ping {
    // todo: implement the conversion of the command into a stream of frames
    /// Convert the command into a stream of frames to be transimitted over the socket
    fn into_stream(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("ping".as_bytes()));

        frame.push_bulk(self.msg.into());
        frame
    }
}

#[allow(dead_code)]
pub struct Get {
    key: String,
}

impl Get {}

#[allow(dead_code)]
pub struct Publish {
    channel: String,
    message: String,
}

impl Publish {}

#[allow(dead_code)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {}

#[allow(dead_code)]
pub struct Subscribe {
    channels: Vec<String>,
}

impl Subscribe {}

#[allow(dead_code)]
pub struct Unsubscribe {
    channels: Vec<String>,
}

impl Unsubscribe {}

#[allow(dead_code)]
pub struct Unknown {
    command: String,
}

impl Unknown {}
