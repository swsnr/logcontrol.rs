default:
    just --list

test-all:
    cargo deny --all-features --locked check
    cargo fmt -- --check
    cargo build --workspace --all-targets --locked
    cargo clippy --workspace --all-targets --locked
    cargo test --workspace --locked
    cargo doc --workspace --locked
