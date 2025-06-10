# Makefile
.PHONY: check fmt clippy audit test quality

# Run all quality checks
quality: fmt clippy audit test

# Format code
fmt:
	cargo fmt --all

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Run clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
audit:
	cargo audit

# Run tests
test:
	cargo test --all-features

# Check compilation
check:
	cargo check --all-targets --all-features

# Clean build artifacts
clean:
	cargo clean