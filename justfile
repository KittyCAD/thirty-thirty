ci:
    just lint
    cargo test --doc
    cargo run --example basic
    cargo nextest run

lint:
    cargo clippy --tests --benches --examples --all-targets

lint-fix:
    cargo clippy --tests --benches --examples --all-targets --fix
