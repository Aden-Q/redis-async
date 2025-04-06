//! Redis commands.
use crate::frame::Frame;

mod hello;
pub use hello::Hello;

mod ping;
pub use ping::Ping;

mod get;
pub use get::Get;

mod getex;
pub use getex::{Expiry, GetEx};

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

mod subscribe;

mod unsubscribe;

/// A trait for all Redis commands.
#[allow(unused)]
pub trait Command: TryInto<Frame, Error = crate::RedisError> {}
