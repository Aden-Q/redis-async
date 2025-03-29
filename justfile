@help
    just -l

@module-tree
    @echo "Showing module tree"
    cargo modules structure --lib

@dependency-tree
    @echo "Showing dependency tree"
    cargo tree

@build-cli
    @echo "Building CLI"
    cargo build --release --bin redis-async-cli

@run-cli
    @echo "Running CLI"
    cargo run --release --bin redis-async-cli

@build-lib
    @echo "Building library"
    cargo build --release --lib
