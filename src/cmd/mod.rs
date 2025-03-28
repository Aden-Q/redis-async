//! Redis commands.
use crate::frame::Frame;

mod ping;
pub use ping::Ping;

mod get;
pub use get::Get;

mod set;
pub use set::Set;

mod del;
pub use del::Del;

mod exists;
pub use exists::Exists;

mod expire;
pub use expire::Expire;

mod ttl;
pub use ttl::Ttl;

mod incr;
pub use incr::Incr;

mod decr;
pub use decr::Decr;

mod lpush;
pub use lpush::LPush;

mod rpush;
pub use rpush::RPush;

mod lpop;
pub use lpop::LPop;

mod rpop;
pub use rpop::RPop;

mod lrange;
pub use lrange::LRange;

mod publish;
pub use publish::Publish;

mod subscribe;
pub use subscribe::Subscribe;

mod unsubscribe;
pub use unsubscribe::Unsubscribe;

/// A trait for all Redis commands.
pub trait Command {
    /// Converts the command into a Frame to be transimitted over the stream.
    fn into_stream(self) -> Frame;
}
