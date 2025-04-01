// use mini_redis::client::{self};
// use tokio_stream::StreamExt;

// async fn publish() -> mini_redis::Result<()> {
//     let mut client = client::connect("127.0.0.1:6379").await?;

//     // publish
//     client.publish("numbers", "1".into()).await?;
//     client.publish("numbers", "two".into()).await?;
//     client.publish("numbers", "3".into()).await?;
//     client.publish("numbers", "four".into()).await?;
//     client.publish("numbers", "5".into()).await?;
//     client.publish("numbers", "six".into()).await?;

//     Ok(())
// }

// async fn subscribe() -> mini_redis::Result<()> {
//     let client = client::connect("127.0.0.1:6379").await?;
//     let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
//     let messages = subscriber
//         .into_stream()
//         .filter(|msg| match msg {
//             Ok(msg) if msg.content.len() == 1 => true,
//             _ => false,
//         })
//         .map(|msg| msg.unwrap().content)
//         .take(3);

//     tokio::pin!(messages);

//     while let Some(msg) = messages.next().await {
//         println!("Got {:?}", msg);
//     }

//     Ok(())
// }

use redis_asyncx::{Client, Result};
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
