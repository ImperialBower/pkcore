.PHONY: clean build test build_test fmt clippy create_docs ayce default help docs test-nightly clippy-nightly nightly tree tree-duplicates deny audit unused-deps install-tools watch install-watch check-wasm check-purity generate-hups-bin test-debug-json nextest heavy marathon mutants mutants-diff coverage coverage-open ci ci-fresh pokerbench-data validate-okf perf-build perf-native perf-report perf-profile perf-check perf-build-all perf-native-all perf-sweep perf-bench

# Default target
default: ayce

# Display help information
help:
	@echo "Available targets:"
	@echo "  make (default)       - Run ayce"
	@echo "  make build           - Build the project"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make test            - Run tests"
	@echo "  make ci              - Mirror GitHub Actions test job (RUSTFLAGS=-Dwarnings)"
	@echo "  make ci-fresh        - Like ci, but 'cargo update' first to match CI's fresh deps"
	@echo "  make nextest         - Run tests with cargo-nextest (installs if missing)"
	@echo "  make heavy           - Run ignored heavy tests"
	@echo "  make marathon        - Run 1000-hand bot marathon stress test"
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
	@echo "  make validate-okf    - Check .okf/ knowledge bundle conformance (OKF v0.1, strict)"
	@echo ""
	@echo "WebAssembly:"
	@echo "  make check-wasm         - Check the library compiles for wasm32-unknown-unknown"
	@echo "  make check-purity       - Assert no rusqlite/zstd/termion/dotenvy with --no-default-features"
	@echo "  make generate-hups-bin  - Generate generated/hups.bin for WASM embedded store"
	@echo ""
	@echo "Datasets:"
	@echo "  make pokerbench-data    - Download the full PokerBench dataset (EPIC-43, ~720MB)"
	@echo ""
	@echo "Tools and Workflow:"
	@echo "  make install-tools   - Install cargo-deny, cargo-udeps, cargo-mutants, and cargo-llvm-cov"
	@echo "  make watch           - Run cargo-watch for check/test loop"
	@echo "  make install-watch   - Install cargo-watch"
	@echo "  make mutants         - Run cargo-mutants on the whole codebase (slow)"
	@echo "  make mutants-diff    - Run cargo-mutants only on files changed vs main"
	@echo "  make coverage        - Generate HTML code coverage report"
	@echo "  make coverage-open   - Generate HTML coverage report and open in browser"
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

# Mirror the GitHub Actions test job exactly: warnings are hard errors and
# incremental compilation is off. Uses the CURRENT Cargo.lock.
ci:
	RUSTFLAGS="-Dwarnings" CARGO_INCREMENTAL=0 cargo test --all

# Like `ci`, but first re-resolves dependencies to the latest compatible
# versions. CI has no committed Cargo.lock and resolves fresh on every run,
# so this is what catches breakage from newer deps (e.g. a method deprecated
# in a point release) before it reaches GitHub.
ci-fresh:
	cargo update
	RUSTFLAGS="-Dwarnings" CARGO_INCREMENTAL=0 cargo test --all

# Run ignored heavy tests
heavy:
	cargo test --test heavy_tests -- --ignored

# Run the 1000-hand bot marathon stress test
marathon:
	cargo test --test bot_marathon -- --include-ignored --nocapture

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
	cargo fmt --all

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

# Validate the .okf/ knowledge bundle against the OKF v0.1 spec (strict: warnings fail too).
# scripts/okf_validate.py is vendored verbatim from the okf skill
# (github.com/scaccogatto/okf-skills, plugin v0.4.0); it carries PEP 723
# metadata, so uv runs it dependency-free. Falls back to python3 + pyyaml.
validate-okf:
	@echo "Validating .okf/ bundle (OKF v0.1, strict)..."
	@if command -v uv >/dev/null 2>&1; then \
		uv run scripts/okf_validate.py .okf --strict; \
	else \
		python3 -c 'import yaml' 2>/dev/null || python3 -m pip install --quiet pyyaml; \
		python3 scripts/okf_validate.py .okf --strict; \
	fi

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

# All You Can Eat - Run all checks at CI strictness.
# Target-specific exports propagate to every prerequisite recipe (build,
# nextest, doc tests, clippy), so warnings become hard errors exactly like
# the GitHub Actions job. Standalone targets (e.g. `make test`) stay lenient.
ayce: export RUSTFLAGS := -Dwarnings
ayce: export CARGO_INCREMENTAL := 0
ayce: fmt build_test clippy create_docs

# Install required tools
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-deny
	cargo install cargo-udeps
	cargo install --locked cargo-mutants
	cargo install cargo-llvm-cov
	rustup component add llvm-tools
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

# Kernel purity gate (AUDIT_Fable_5.md III.1 / III.6.1): assert that with default
# features off, the storage/terminal layers drop out and no rusqlite/zstd/termion/
# dotenvy remains in the dependency tree. serde_yaml_bw is a documented exception
# (it arrives transitively via pkstate). This target is the single source of the
# purity gate — the CI job in basic.yaml invokes `make check-purity` rather than
# re-inlining the pipeline (audit P9j.4). The `::error::` prefix is a GitHub
# Actions annotation in CI and a harmless plain line locally.
check-purity:
	@leaked=$$(cargo tree --no-default-features -e no-dev | grep -iE 'rusqlite|zstd|termion|dotenvy' || true); \
	if [ -n "$$leaked" ]; then \
		echo "::error::Purity gate failed — these deps must be feature-gated behind store/terminal:"; \
		echo "$$leaked"; \
		exit 1; \
	fi; \
	echo "Purity gate passed: no rusqlite/zstd/termion/dotenvy with --no-default-features."; \
	echo "(serde_yaml_bw remains via pkstate — documented ceiling, see AUDIT_Fable_5.md III.1.)"

# Generate the embedded HUP binary store for WASM builds
generate-hups-bin:
	cargo run --example export_hups_bin

# Download the full PokerBench dataset (EPIC-43) — both the test and train
# splits, CSV + JSON (~720 MB total) — into POKERBENCH_DATA_DIR (default
# ./data/pokerbench, which is gitignored). Re-runnable: existing files are
# skipped, so an interrupted run can simply be re-invoked. Used by the
# `pokerbench` feature's loaders and the ignored real-data integration tests.
pokerbench-data:
	@dir="$${POKERBENCH_DATA_DIR:-data/pokerbench}"; \
	base="https://huggingface.co/datasets/RZ412/PokerBench/resolve/main"; \
	mkdir -p "$$dir"; \
	for f in \
		preflop_1k_test_set_game_scenario_information.csv \
		preflop_1k_test_set_prompt_and_label.json \
		preflop_60k_train_set_game_scenario_information.csv \
		preflop_60k_train_set_prompt_and_label.json \
		postflop_10k_test_set_game_scenario_information.csv \
		postflop_10k_test_set_prompt_and_label.json \
		postflop_500k_train_set_game_scenario_information.csv \
		postflop_500k_train_set_prompt_and_label.json ; do \
		if [ -f "$$dir/$$f" ]; then \
			echo "exists, skipping: $$f"; \
		else \
			echo "downloading:    $$f"; \
			curl -fL --progress-bar -o "$$dir/$$f" "$$base/$$f" \
				|| { echo "FAILED: $$f"; rm -f "$$dir/$$f"; exit 1; }; \
		fi; \
	done; \
	echo "PokerBench dataset ready in $$dir"

# Run mutation testing on the full codebase (slow — can take hours)
mutants:
	@if ! cargo mutants --version >/dev/null 2>&1; then \
		echo "cargo-mutants is not installed."; \
		printf "Would you like to install it now? [y/N] "; \
		read answer; \
		if [ "$$answer" = "y" ] || [ "$$answer" = "Y" ]; then \
			cargo install --locked cargo-mutants; \
		else \
			echo "Skipping. Run 'cargo install cargo-mutants' to install manually."; \
			exit 1; \
		fi; \
	fi
	cargo mutants

# Run mutation testing only on files changed vs main (faster, good before pushing)
mutants-diff:
	@if ! cargo mutants --version >/dev/null 2>&1; then \
		echo "cargo-mutants is not installed. Run 'make install-tools' first."; \
		exit 1; \
	fi
	git diff main..HEAD > /tmp/pkcore-diff.txt
	cargo mutants --in-diff /tmp/pkcore-diff.txt

# Generate HTML code coverage report using cargo-llvm-cov
coverage:
	@if ! cargo llvm-cov --version >/dev/null 2>&1; then \
		echo "cargo-llvm-cov is not installed."; \
		printf "Would you like to install it now? [y/N] "; \
		read answer; \
		if [ "$$answer" = "y" ] || [ "$$answer" = "Y" ]; then \
			cargo install cargo-llvm-cov; \
			rustup component add llvm-tools; \
		else \
			echo "Skipping. Run 'cargo install cargo-llvm-cov && rustup component add llvm-tools' to install manually."; \
			exit 1; \
		fi; \
	fi
	cargo llvm-cov --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

# Generate HTML coverage report and open in browser
coverage-open: coverage
	@COV_PATH="./target/llvm-cov/html/index.html"; \
	if command -v xdg-open >/dev/null 2>&1; then \
		xdg-open "$$COV_PATH"; \
	elif command -v open >/dev/null 2>&1; then \
		open "$$COV_PATH"; \
	else \
		echo "No supported opener found (tried xdg-open and open)."; \
		echo "Open $$COV_PATH manually."; \
		exit 1; \
	fi


# ---------------------------------------------------------------------------
# Performance harness (docs/superpowers/specs/2026-07-30-kernel-performance-
# harness-design.md). The perf crate is its own workspace root, so these
# targets cd into perf/ rather than using the root cargo invocation.
# ---------------------------------------------------------------------------
PKCORE_VERSION := $(shell grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
PERF_BIN := perf/target/release/perf
PERF_TARGET_DIR_ALL := target/all-features
PERF_BIN_ALL := perf/$(PERF_TARGET_DIR_ALL)/release/perf

# Phony, not a file target keyed on $(PERF_BIN): a bare `$(PERF_BIN):` rule
# with no prerequisites only reruns `cargo build` when the binary file is
# absent, so any perf/src edit was silently ignored by perf-native,
# perf-report, and perf-profile once the binary existed once. `cargo build`
# already does its own cheap up-to-date check; let it, rather than relying on
# make's file-timestamp tracking to decide whether a rebuild is needed.
.PHONY: perf-build
perf-build:
	cd perf && cargo build --release

# Measure the pure kernel on this host and write a results file.
perf-native: perf-build
	PKCORE_VERSION=$(PKCORE_VERSION) $(PERF_BIN) run \
		--utc "$$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Regenerate docs/perf/RESULTS.md from every committed results file.
perf-report: perf-build
	$(PERF_BIN) report

# Profile one workload with samply. Requires: cargo install samply
# Usage: make perf-profile WORKLOAD=eval.seven.hand_rank_value
# Uses the all-features binary: its catalog is a superset of the pure-kernel
# one, so every workload in docs/perf/RESULTS.md — including equity.* and
# dealeval.* — is profilable. The pure-kernel binary's catalog omits them,
# and `perf run <name>` inside samply then dies with "no workload matched".
perf-profile: perf-build-all
	@if [ -z "$(WORKLOAD)" ]; then \
		echo "usage: make perf-profile WORKLOAD=<name>  (see: $(PERF_BIN_ALL) list)"; \
		exit 1; \
	fi
	samply record $(PERF_BIN_ALL) run $(WORKLOAD) --trials 50 --stdout

# Build the perf runner with every workload feature enabled, into a distinct
# target directory from the pure-kernel build. Previously both builds shared
# perf/target/release, so `perf-build-all` silently overwrote the pure-kernel
# binary; a later `perf-native` (or `perf-report`, or `perf-profile`) would
# then run the all-features binary under the pure-kernel label, writing
# results whose `features` did not match what actually produced them.
perf-build-all:
	cd perf && cargo build --release --target-dir $(PERF_TARGET_DIR_ALL) --features "equity sim"

# Measure everything, all features on. Labelled so it sits alongside the
# pure-kernel run from `make perf-native` rather than overwriting it.
perf-native-all: perf-build-all
	PKCORE_VERSION=$(PKCORE_VERSION) $(PERF_BIN_ALL) run \
		--label all-features \
		--utc "$$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Rayon pool-size sweep over the parallel workloads (design Section 5).
perf-sweep: perf-build-all
	PKCORE_VERSION=$(PKCORE_VERSION) $(PERF_BIN_ALL) run \
		--sweep --label sweep \
		--utc "$$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# The same nano workloads under Criterion and Divan, for side-by-side
# comparison with the custom harness (`make perf-native`). Educational, not a
# gate — see docs/perf/HARNESS_COMPARISON.md.
perf-bench:
	cd perf && cargo bench --bench criterion
	cd perf && cargo bench --bench divan

# Lint and test the perf crate. It sits outside `make ayce`, so this keeps it
# from rotting.
perf-check:
	cd perf && cargo fmt --check
	cd perf && cargo clippy --all-targets -- -D warnings
	cd perf && cargo clippy --all-targets --features "equity sim" -- -D warnings
	cd perf && cargo build --bins --features "equity sim"
	cd perf && cargo test
	cd perf && cargo test --features "equity sim"
