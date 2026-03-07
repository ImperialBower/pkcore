.PHONY: clean build test build_test fmt clippy create_docs ayce default help docs

# Default target
default: ayce

# Display help information
help:
	@echo "Available targets:"
	@echo "  make build         - Build the project"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make test          - Run tests"
	@echo "  make build_test    - Clean once, then build and test"
	@echo "  make fmt           - Format code"
	@echo "  make clippy        - Run clippy linter"
	@echo "  make create_docs   - Create documentation"
	@echo "  make docs          - Create docs and open in browser (macOS/Linux)"
	@echo "  make ayce          - Run all checks (fmt → build_test → clippy → create_docs)"
	@echo "  make help          - Display this help message"

# Clean build artifacts
clean:
	cargo clean

# Build the project
build:
	cargo build

# Run tests
test:
	cargo test

# Clean once, then run build + test
build_test: clean build test

# Format code
fmt:
	cargo fmt

# Run clippy linter
clippy:
	cargo clippy -- -W clippy::pedantic

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
