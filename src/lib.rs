mod connection;
pub use connection::Connection;

mod db;

mod frame;
// re-export Frame
pub use frame::Frame;

mod cmd;
pub use cmd::Command;

mod client;
pub use client::Client;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
