# redis-async

An asynchronous Redis client library and a Redis CLI built in Rust, compliant with RESP 3 (Redis Serialization Protocol)

## Usage

### Using the lib

### Using the CLI

## Connection pooling

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
