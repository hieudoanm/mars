SHELL := /bin/sh

.PHONY: all fmt check clippy test test-runtime test-jupyter coverage build desktop-build e2e benchmark clean install-cli run-examples

COVERAGE_PACKAGES := mars-core mars-document mars-runtime mars-runtime-evcxr \
	mars-runtime-julia mars-runtime-python mars-runtime-r mars-runtime-cpp \
	mars-protocol mars-kernel mars-mcp mars-jupyter mars-renderer mars-lsp mars
COVERAGE_MIN_LINES := 80
COVERAGE_MIN_FUNCTIONS := 80

all: check desktop-build

fmt:
	cargo fmt --all --check

check: fmt clippy test

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

test-runtime:
	cargo test -p mars-runtime

test-jupyter:
	cargo test -p mars-jupyter

coverage:
	cargo llvm-cov $(COVERAGE_PACKAGES:%=-p %) --all-targets \
		--fail-under-lines $(COVERAGE_MIN_LINES) \
		--fail-under-functions $(COVERAGE_MIN_FUNCTIONS)

build:
	cargo build --workspace

desktop-build:
	cd packages/desktop && npm run build

e2e:
	cd packages/desktop && npm run test:e2e

benchmark:
	cargo run -p mars-runtime --release --bin runtime_benchmark -- $${CELLS:-8}

clean:
	cargo clean

install-cli:
	cargo install --path crates/mars-cli --force

run-examples:
	cd examples/mars && ./run.sh
