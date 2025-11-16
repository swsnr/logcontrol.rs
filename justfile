default:
    just --list

test-all:
    cargo +stable deny --all-features --locked check
    cargo +stable fmt -- --check
    cargo +stable build --workspace --all-targets --locked
    cargo +stable clippy --workspace --all-targets --locked
    cargo +stable test --workspace --locked
    cargo +stable doc --workspace --locked
