.PHONY: build install release test clean check clippy fmt

build:
	cargo build --release

install: build
	./scripts/install-local.sh

release: install
	@echo "SENA $$(./target/release/sena --version) installed and ready"

test:
	cargo test

check:
	cargo check

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean

all: fmt clippy test build install
