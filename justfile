# list all receipts
@help:
  just -l

@module-tree:
    echo "Showing module tree"
    cargo modules structure --lib

@dependency-tree:
    echo "Showing dependency tree"
    cargo tree

@format:
    echo "Formatting code"
    cargo fmt --all -- --check

@lint:
    echo "Linting code"
    cargo clippy --all --examples --tests --benches -- -D warnings

@fix:
    echo "Fixing code"
    cargo fix --all --allow-dirty

@test:
    echo "Running tests"
    cargo test

@build-cli:
    echo "Building CLI"
    cargo build --release --bin redis-async-cli

@run-cli:
    echo "Running CLI"
    cargo run --release --bin redis-async-cli

@build-lib:
    echo "Building library"
    cargo build --release --lib
