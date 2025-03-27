use redis_async::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:6379").await?;

    let resp = client.set("mykey", "myvalue").await?.unwrap();

    println!("SET command response: {}", resp);

    let resp = client.get("mykey").await?;

    if let Some(data) = resp {
        println!("GET command response: {}", data);
    } else {
        println!("Key not found");
    }

    Ok(())
}
