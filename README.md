# redis-async

An asynchronous Redis client library and a Redis CLI built in Rust, compatible with RES (Redis Serialization Protocol) version 2 and 3, built with [Tokio][1]. Inspired by [mini-redis][2].

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

You can install the CLI as a binary or run it with [Cargo][3].

To install as a binary into `~/.cargo/bin`:

```shell
> cargo install --path .
```

Then you can run it:

```shell
> redis-async-cli
```

To build and run without installation:

```shell
> cargo build --release --bin redis-async-cli
```

Then you can it:

```shell
> ./target/release/redis-async-cli
```

To use the CLI, you first need to run a Redis server. Then you can run this CLI in either interactive mode or command line mode:

+ Interactive mode:
+ Command line mode:

## TLS/SSL

TBD. Not available yet.

## Connection pooling

TBD. Not available yet.

## RESP2/RESP3

## Supported commands

This library is more on prototype. More commands will be added later on.

+ [PING][4]
+ [GET][5]
+ [SET][6]
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

### Local build

To build the lib:

```shell
> cargo build --lib
```

To build the CLI:

```shell
> cargo build --bin redis-async-cli
```

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
[3]: https://github.com/rust-lang/cargo
[4]: https://redis.io/docs/latest/commands/ping/
[5]: https://redis.io/docs/latest/commands/get/
[6]: https://redis.io/docs/latest/commands/set/
