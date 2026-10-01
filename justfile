ci:
    just lint
    cargo nextest run
    cargo test --doc

lint:
    cargo clippy --tests --benches --examples --all-targets

lint-fix:
    cargo clippy --tests --benches --examples --all-targets --fix
