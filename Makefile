# Commandes de développement de wikik (voir CLAUDE.md).
.PHONY: check fmt build release

check:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

fmt:
	cargo fmt --all

build:
	cargo build --workspace

release:
	cargo build --release -p wikik
