use my_redis::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut c = Client::connect("127.0.0.1:6379").await?;

    let resp = c.ping(None).await?;

    println!("Pinged the Redis server. Got response: {:?}", resp);

    // let mut client = client::connect("127.0.0.1:6379").await?;

    // client.set("hello", "world".into()).await?;

    // let result = client.get("hello").await?;

    // println!("got value from the server; result={:?}", result);

    Ok(())
}
