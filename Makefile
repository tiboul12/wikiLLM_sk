# Commandes de développement de wikik (voir CLAUDE.md).
.PHONY: check fmt build front release install types

check:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

fmt:
	cargo fmt --all

build:
	cargo build --workspace

# Types TypeScript de l'API, générés depuis Rust (ts-rs) dans front/src/types/.
types:
	cargo test -q -p wikik-core export_bindings

# Front Svelte compilé dans front/dist/, embarqué ensuite par le binaire.
front: types
	cd front && npm ci && npm run build

release: front
	cargo build --release -p wikik

# Installe le binaire dans ~/.cargo/bin.
install: front
	cargo install --path crates/wikik --locked
