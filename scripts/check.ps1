$ErrorActionPreference = 'Stop'
pnpm check:version
pnpm format:check
pnpm check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
