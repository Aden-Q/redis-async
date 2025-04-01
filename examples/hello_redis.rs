use redis_asyncx::{Client, Result};

use std::str;
use tokio::task::JoinHandle;

// simulate 10 clients to ping the redis server asynchronously
// we use a single-thread tokio runtime and an async sleep to verify
// the client can send requests concurrently without blocking
#[tokio::main(worker_threads = 1)]
async fn main() -> Result<()> {
    let num_clients = 10;
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(num_clients);

    for id in 0..num_clients {
        let handle = tokio::spawn(async move {
            let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
            let response = client.ping(Some("Redis".as_bytes())).await.unwrap();

            if let Ok(string) = str::from_utf8(&response) {
                println!("From client {id}, got: \"{}\"", string);
            } else {
                println!("From client {id}, got: {:?}", response);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
