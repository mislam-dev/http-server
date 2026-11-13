
# ================================
# Development
# ================================

start:
    cargo run

dev:
    cargo watch -c -w src -d 0.1 -x run

test:
    cargo test
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

check:
    cargo check

fmt:
    cargo fmt

lint:
    cargo clippy --all-targets --all-features
ci:
    cargo fmt --check
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test