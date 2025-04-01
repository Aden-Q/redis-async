use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use redis_asyncx::Client;
use std::process::Command; // Run programs
use testcontainers::{
    GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

use tokio::sync::OnceCell;

type TestResult = Result<(), Box<dyn std::error::Error + 'static>>;

const REDIS_PORT: u16 = 6379;

static REDIS_CONTAINER: OnceCell<testcontainers::ContainerAsync<GenericImage>> =
    OnceCell::const_new();

async fn setup_redis() -> &'static testcontainers::ContainerAsync<GenericImage> {
    let container = REDIS_CONTAINER
        .get_or_init(|| async {
            GenericImage::new("redis", "7.2.4")
                .with_exposed_port(REDIS_PORT.tcp())
                .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
                .start()
                .await
                .unwrap()
        })
        .await
        .to_owned();

    // wait until the Redis server is ready
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    container
}

#[tokio::test]
async fn redis_async_cli_ping() -> TestResult {
    let container = setup_redis().await;

    let host = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(REDIS_PORT).await?;

    let mut cmd = Command::cargo_bin("redis-async-cli").unwrap();

    cmd.args([
        "--host",
        &host.to_string(),
        "--port",
        &host_port.to_string(),
    ]);

    cmd.arg("ping");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PONG"))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[tokio::test]
async fn redis_async_cli_set_get() -> TestResult {
    let container = setup_redis().await;

    let host = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(REDIS_PORT).await?;

    let mut cmd = Command::cargo_bin("redis-async-cli").unwrap();

    cmd.args([
        "--host",
        &host.to_string(),
        "--port",
        &host_port.to_string(),
    ]);

    cmd.arg("set").arg("mykey").arg("myvalue");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("OK"))
        .stderr(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("redis-async-cli").unwrap();

    cmd.args([
        "--host",
        &host.to_string(),
        "--port",
        &host_port.to_string(),
    ]);

    cmd.arg("get").arg("mykey");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"myvalue\""))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[tokio::test]
async fn redis_client_ping() -> TestResult {
    let container = setup_redis().await;

    let host = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(REDIS_PORT).await?;

    let mut client = Client::connect(format!("{}:{}", host, host_port)).await?;

    let response = client.ping(None).await?;

    if let Ok(string) = std::str::from_utf8(&response) {
        assert_eq!(string, "PONG");
    } else {
        panic!("Invalid response: {:?}", response);
    }

    Ok(())
}

#[tokio::test]
async fn redis_client_set_get() -> TestResult {
    let container = setup_redis().await;

    let host = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(REDIS_PORT).await?;

    let mut client = Client::connect(format!("{}:{}", host, host_port)).await?;

    let response: Option<Vec<u8>> = client.set("mykey", "myvalue".as_bytes()).await?;

    if let Some(value) = response {
        if let Ok(string) = std::str::from_utf8(&value) {
            assert_eq!(string, "OK");
        } else {
            panic!("Invalid response: {:?}", value);
        }
    } else {
        panic!("No response");
    }

    let response = client.get("mykey").await?;
    if let Some(value) = response {
        if let Ok(string) = std::str::from_utf8(&value) {
            assert_eq!(string, "myvalue");
        } else {
            panic!("Invalid response: {:?}", value);
        }
    } else {
        panic!("No response");
    }

    Ok(())
}
