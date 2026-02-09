.PHONY: check fmt clippy test build ci

# Run all CI checks locally (same as GitHub Actions)
ci: fmt clippy test build
	@echo "✅ All CI checks passed!"

# Check formatting (same as CI)
fmt:
	cargo fmt --check

# Run clippy with warnings as errors (same as CI)
clippy:
	cargo clippy -- -D warnings

# Run all tests
test:
	cargo test

# Release build
build:
	cargo build --release

# Auto-fix formatting
fix:
	cargo fmt
	@echo "Formatting fixed."

# Alias: check = ci
check: ci
