set windows-shell := ["pwsh.exe", "-Command"]

init:
	cargo install cargo-binstall
	cargo binstall cargo-insta cargo-nextest cargo-wasi

build:
	cargo build --bin rex --no-default-features

build-wasm:
	cd plugins && cargo build --workspace --target wasm32-wasip1 --release

check:
	cargo check --workspace

format:
	cargo fmt --all

format-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets

lint-wasm:
	cd plugins && cargo clippy --workspace --all-targets

mcp:
	REX_NPM_VERSION=* npx @modelcontextprotocol/inspector -- cargo run -- mcp

test name="":
	just build
	cargo nextest run --workspace {{name}}

test-ci:
	cargo nextest run --workspace --exclude rex_pdk --profile ci --config-file ./.cargo/nextest.toml
