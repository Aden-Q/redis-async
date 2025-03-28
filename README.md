# redis-async

An asynchronous Redis client library and a Redis CLI built in Rust, compliant with RESP 3 (Redis Serialization Protocol)

## Usage

### Using the lib

First import the dependency:

```TOML
# in Cargo.toml
[dependencies.redis_async]
git = "https://github.com/aden-q/redis-async.git"
```

Then use the lib in your Rust code:

```Rust
use redis_async::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:6379").await?;

    let _ = client.ping("Hello, Redis!").await?.unwrap();

    let _ = client.set("mykey", "myvalue").await?.unwrap();

    let _ = client.get("mykey").await?;

    Ok(())
}
```

### Using the CLI

TBD. Not available yet.

## TLS/SSL

TBD. Not available yet.

## Connection pooling

TBD. Not available yet.

## Supported commands

This library is more on prototype. More commands will be added later on.

+ PING
+ GET
+ SET
+ DEL
+ EXISTS
+ EXPIRE
+ TTL
+ INCR
+ DECR
+ LPUSH
+ RPUSH
+ LPOP
+ RPOP
+ LRANGE

## Development

TBD. Thinking of which may people prefer if they don't want to install Redis on their local.

Also due to gotchas from different RESP versions and Redis versions. A local dev may be necessary to replicate the same results on different platform.

### Docs

```shell
> cargo doc --no-deps --open
```

## License

The project is licensed under the [MIT license](./LICENSE).
