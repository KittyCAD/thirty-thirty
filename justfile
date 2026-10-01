clippy-flags := "--workspace --tests --benches --examples --all-targets"

# Check most of CI, but locally.
@ci:
    just lint
    just test
    just run-examples
    just fmt-check

test:
    cargo nextest run
    cargo test --doc

lint:
    cargo clippy {{clippy-flags}} -- -D warnings

# Fix some lints automatically.
lint-fix:
    cargo clippy {{clippy-flags}} --fix

run-examples:
    cargo run --example basic

# Run unit tests, output coverage to `lcov.info`.
test-with-coverage:
    cargo llvm-cov nextest --all-features --release --workspace --lcov --output-path lcov.info

# Check formatting and typos.
fmt-check:
    cargo fmt --check
    cargo sort --check
    typos

publish version:
    cargo publish -p thirty-thirty --dry-run
    git tag {{version}}
    git push --tags
    cargo publish -p thirty-thirty

