use redis_async::{Client, Result};
use std::str;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:6379").await?;
    let response: Option<Vec<u8>> = client.set("mykey", "myvalue".as_bytes()).await?;

    if let Some(value) = response {
        if let Ok(string) = str::from_utf8(&value) {
            println!("{}", string);
        } else {
            println!("{:?}", value);
        }
    } else {
        println!("(nil)");
    }

    let response = client.get("mykey").await?;
    if let Some(value) = response {
        if let Ok(string) = str::from_utf8(&value) {
            println!("\"{}\"", string);
        } else {
            println!("{:?}", value);
        }
    } else {
        println!("(nil)");
    }

    let resp = client.del(vec!["mykey"]).await?;

    println!("DEL command response: {}", resp);

    let resp = client.exists(vec!["mykey"]).await?;

    println!("EXISTS command response: {}", resp);

    Ok(())
}
