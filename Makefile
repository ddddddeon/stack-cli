NAME=stack-cli

.PHONY: run
run: lint
	cargo run

.PHONY: build
build: lint
	cargo build

.PHONY: test
test:
	cargo test

.PHONY: release
release: lint
	cargo build --release

.PHONY: watch
watch:
	cargo watch -x "clippy; cargo run"

.PHONY: clean
clean:
	cargo clean

.PHONY: install
install:
	mv target/release/$(NAME) /usr/bin/

.PHONY: publish
publish:
	cargo publish

.PHONY: fmt
fmt:
	rustfmt **/*.rs

.PHONY: lint
lint:
	cargo clippy
