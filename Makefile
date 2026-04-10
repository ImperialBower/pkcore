.PHONY: clean build test build_test fmt clippy create_docs ayce default help docs test-nightly clippy-nightly nightly tree tree-duplicates deny audit unused-deps install-tools watch install-watch check-wasm generate-hups-bin test-debug-json nextest

# Default target
default: ayce

# Display help information
help:
	@echo "Available targets:"
	@echo "  make (default)       - Run ayce"
	@echo "  make build           - Build the project"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make test            - Run tests"
	@echo "  make nextest         - Run tests with cargo-nextest (installs if missing)"
	@echo "  make test-debug-json - Run tests with debug-json feature (save/load use JSON)"
	@echo "  make build_test      - Clean, build, nextest, and doc tests"
	@echo "  make fmt             - Format code"
	@echo "  make clippy          - Run clippy linter"
	@echo "  make create_docs     - Build documentation"
	@echo "  make docs            - Build docs and open in browser"
	@echo "  make ayce            - Run fmt, build_test, clippy, and docs"
	@echo "  make help            - Display this help message"
	@echo ""
	@echo "Nightly:"
	@echo "  make test-nightly    - Run all tests with nightly"
	@echo "  make clippy-nightly  - Run clippy with nightly and deny warnings"
	@echo "  make nightly         - Run nightly test and clippy checks"
	@echo "  make unused-deps     - Find unused dependencies with cargo-udeps"
	@echo ""
	@echo "Dependencies and Security:"
	@echo "  make tree            - Show dependency tree"
	@echo "  make tree-duplicates - Show duplicate dependencies"
	@echo "  make deny            - Run full cargo-deny checks"
	@echo "  make audit           - Run advisory-only security audit"
	@echo ""
	@echo "WebAssembly:"
	@echo "  make check-wasm         - Check the library compiles for wasm32-unknown-unknown"
	@echo "  make generate-hups-bin  - Generate generated/hups.bin for WASM embedded store"
	@echo ""
	@echo "Tools and Workflow:"
	@echo "  make install-tools   - Install cargo-deny and cargo-udeps"
	@echo "  make watch           - Run cargo-watch for check/test loop"
	@echo "  make install-watch   - Install cargo-watch"
	@echo ""

# Clean build artifacts
clean:
	cargo clean

# Build the project
build:
	cargo build

# Run tests
test:
	cargo test

# Run tests with cargo-nextest (installs if not present)
nextest:
	@if ! cargo nextest --version >/dev/null 2>&1; then \
		echo "cargo-nextest is not installed."; \
		printf "Would you like to install it now? [y/N] "; \
		read answer; \
		if [ "$$answer" = "y" ] || [ "$$answer" = "Y" ]; then \
			cargo install --locked cargo-nextest; \
		else \
			echo "Skipping. Run 'cargo install cargo-nextest' to install manually."; \
			exit 1; \
		fi; \
	fi
	cargo nextest run

# Run tests with the debug-json feature enabled (SolverResult::save/load use JSON)
test-debug-json:
	cargo test --features debug-json

# Clean once, then build, run nextest, and run doc tests
build_test: clean build nextest
	cargo test --doc

# Format code
fmt:
	cargo fmt

# Run clippy linter
clippy:
	cargo clippy -- -W clippy::pedantic

test-nightly:
	cargo +nightly test --all-targets --all-features

clippy-nightly:
	cargo +nightly clippy --lib --all-features -- -D warnings

nightly: test-nightly clippy-nightly

# Show dependency tree
tree:
	@echo "Showing dependency tree..."
	cargo tree --workspace

# Show duplicate dependencies
tree-duplicates:
	@echo "Showing duplicate dependencies..."
	cargo tree --workspace --duplicates

# Security checks with cargo-deny
deny:
	@echo "Running cargo-deny checks..."
	cargo deny check

# Security audit with cargo-deny (advisories only)
audit:
	@echo "Running security audit..."
	cargo deny check advisories

# Check for unused dependencies (requires nightly)
unused-deps:
	@echo "Checking for unused dependencies..."
	cargo +nightly udeps --workspace --all-features

# Create documentation
create_docs:
	cargo doc --no-deps

# Open documentation in browser
docs: create_docs
	@DOC_PATH="./target/doc/pkcore/index.html"; \
	if command -v xdg-open >/dev/null 2>&1; then \
		xdg-open "$$DOC_PATH"; \
	elif command -v open >/dev/null 2>&1; then \
		open "$$DOC_PATH"; \
	else \
		echo "No supported opener found (tried xdg-open and open)."; \
		echo "Open $$DOC_PATH manually."; \
		exit 1; \
	fi

# All You Can Eat - Run all checks
ayce: fmt build_test clippy create_docs

# Install required tools
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-deny
	cargo install cargo-udeps
	@echo ""
	@echo "✓ Tools installed!"
	@echo ""

# Watch mode for development (requires cargo-watch)
watch:
	cargo watch -x "check --workspace" -x "test --workspace"

# Install cargo-watch
install-watch:
	cargo install cargo-watch

# Check that the library compiles for WebAssembly
check-wasm:
	cargo check --target wasm32-unknown-unknown

# Generate the embedded HUP binary store for WASM builds
generate-hups-bin:
	cargo run --example export_hups_bin

