.PHONY: build build-rust build-ts test test-rust test-ts coverage clean changelog

# ───────────────────────────────────────────────────────────────────────
# Build

build: build-rust build-ts

build-rust:
	cargo build --release

build-ts:
	cd plugin && bun run build

# ───────────────────────────────────────────────────────────────────────
# Test

test: test-rust test-ts

test-rust:
	cargo test --all

test-ts:
	cd plugin && bun test

# ───────────────────────────────────────────────────────────────────────
# Coverage (requires cargo-llvm-cov: cargo install cargo-llvm-cov)

coverage:
	cargo llvm-cov --all --summary-only

clean:
	cargo clean
	rm -rf target plugin/dist

# ───────────────────────────────────────────────────────────────────────
# Release
#
# Regenerate CHANGELOG.md from Conventional Commits. Pass the tag you are
# about to cut so its section is dated instead of "Unreleased":
#   make changelog TAG=v0.2.2

changelog:
	npx -y git-cliff@2 $(if $(TAG),--tag $(TAG)) -o CHANGELOG.md
