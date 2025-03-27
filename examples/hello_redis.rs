use redis_async::{Client, Result};

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
            let mut c = Client::connect("127.0.0.1:6379").await.unwrap();
            let resp = c.ping(Some("Redis")).await.unwrap();
            // sleep for 1 second, this should not block other clients
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            println!("From client {id} Got response: {resp}");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
