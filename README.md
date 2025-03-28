# redis-async

An asynchronous Redis client library and a Redis CLI built in Rust, compliant with RES (Redis Serialization Protocol) 2 and 3, built with [Tokio][1].
Inspired by [mini-redis][2].

## Usage

### Using the lib

First import dependencies:

```TOML
# in Cargo.toml
[dependencies.redis-async]
git = "https://github.com/aden-q/redis-async.git"
```

Then use the lib in your Rust code:

```Rust
use redis_async::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("127.0.0.1:6379").await?;

    let _ = client.ping(Some("Hello, Redis!")).await?;

    let _ = client.set("mykey", "myvalue").await?;

    let _ = client.get("mykey").await?;

    Ok(())
}
```

More examples can be found in the [examples](./examples/) directory.

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

Also due to gotchas from different RESP versions and Redis versions. A local dev may be necessary to for reproducible build and test environment.

### Docs

```shell
> cargo doc --no-deps --open
```

## License

The project is licensed under the [MIT license](./LICENSE).

[1]: https://tokio.rs/
[2]: https://github.com/tokio-rs/mini-redis
