ci:
    cargo clippy --tests --benches --examples --all-targets
    cargo nextest run
    cargo test --doc
