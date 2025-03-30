# list all receipts
@help:
  just -l

# show module tree
@module-tree:
    echo "Showing module tree"
    cargo modules structure --lib

# show dependency tree
@dependency-tree:
    echo "Showing dependency tree"
    cargo tree

# format using rustfmt
@format:
    echo "Formatting code"
    cargo fmt --all -- --check

# lint code using clippy
@lint:
    echo "Linting code"
    cargo clippy --all --examples --tests --benches -- -D warnings

# run cargo fix
@fix:
    echo "Fixing code"
    cargo fix --all --allow-dirty

# run all test suites
@test:
    echo "Running tests"
    cargo test --all

# build the cli
@build-cli:
    echo "Building CLI"
    cargo build --release --bin redis-async-cli

# run the cli
@run-cli:
    echo "Running CLI"
    cargo run --release --bin redis-async-cli

# build the library
@build-lib:
    echo "Building library"
    cargo build --release --lib
