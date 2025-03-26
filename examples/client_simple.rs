// use bytes::Bytes;
// use mini_redis::client;
// use tokio::sync::{mpsc, oneshot};

// type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

// #[derive(Debug)]
// enum Command {
//     Get {
//         key: String,
//         resp: Responder<Option<Bytes>>,
//     },
//     Set {
//         key: String,
//         val: Bytes,
//         resp: Responder<()>,
//     },
// }

#[tokio::main]
async fn main() {
    // let (tx, mut rx) = mpsc::channel(32);
    // let tx2 = tx.clone();

    // // simulate 1 client tosend a Get request to redis server
    // let sender_task1 = tokio::spawn(async move {
    //     let (resp_tx, resp_rx) = oneshot::channel();

    //     let cmd = Command::Get {
    //         key: "foo".to_string(),
    //         resp: resp_tx,
    //     };

    //     println!("Sending from first handle: {:?}", cmd);

    //     tx.send(cmd).await.unwrap();

    //     let res: Result<
    //         Result<Option<Bytes>, Box<dyn Error + Send + Sync>>,
    //         oneshot::error::RecvError,
    //     > = resp_rx.await;
    //     println!("Got: {:?}", res);
    // });

    // // simulate 1 client tosend a Set request to redis server
    // let sender_task2 = tokio::spawn(async move {
    //     let (resp_tx, resp_rx) = oneshot::channel();

    //     let cmd = Command::Set {
    //         key: "foo".to_string(),
    //         val: "bar".into(),
    //         resp: resp_tx,
    //     };

    //     println!("Sending from second handle: {:?}", cmd);

    //     tx2.send(cmd).await.unwrap();

    //     let res = resp_rx.await;
    //     println!("Got: {:?}", res);
    // });

    // // spawn a task coordinator to proxy between client and server
    // let manager = tokio::spawn(async move {
    //     let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    //     while let Some(msg) = rx.recv().await {
    //         use Command::*;

    //         match msg {
    //             Get { key, resp } => {
    //                 let res = client.get(&key).await;
    //                 let _ = resp.send(res);
    //             }
    //             Set { key, val, resp } => {
    //                 let res = client.set(&key, val).await;
    //                 let _ = resp.send(res);
    //             }
    //         };
    //     }
    // });

    // let (res, ..) = tokio::join!(sender_task1, sender_task2, manager);
    // res.unwrap();
}
