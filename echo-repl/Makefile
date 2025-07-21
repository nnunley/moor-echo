.PHONY: help build test clean doc fmt lint bench release install all check

# Default target
all: fmt lint test

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the project
	cargo build --all-features

test: ## Run all tests (Rust + Echo)
	cargo test --all-features
	@if [ -x ./run_echo_tests.py ]; then \
		echo "Running Echo language tests..."; \
		./run_echo_tests.py; \
	fi

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

doc: ## Generate and open documentation
	cargo doc --all-features --no-deps --open

fmt: ## Format code
	cargo fmt

lint: ## Run clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

bench: ## Run benchmarks
	cargo bench --all-features

release: ## Build optimized release binary
	cargo build --release --all-features

install: ## Install the binary locally
	cargo install --path . --force

check: ## Run all checks (fmt, lint, test)
	@echo "Checking formatting..."
	cargo fmt -- --check
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Running tests..."
	cargo test --all-features

# Echo language testing
test-echo: ## Run only Echo language tests
	@./run_echo_tests.py

test-rust: ## Run only Rust unit tests
	cargo test --all-features

test-file: ## Run specific Echo test file (usage: make test-file FILE=path/to/test.echo)
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make test-file FILE=path/to/test.echo"; \
		exit 1; \
	fi
	@./run_echo_tests.py $(FILE)

repl: ## Run the Echo REPL
	cargo run

# Development helpers
watch: ## Watch for changes and run tests
	cargo watch -x test

coverage: ## Generate test coverage report
	cargo tarpaulin --all-features --out html
	@echo "Coverage report generated at target/tarpaulin/tarpaulin-report.html"

update: ## Update dependencies
	cargo update
	cargo upgrade --dry-run

# Feature-specific builds
build-jit: ## Build with JIT support
	cargo build --features jit

build-wasm: ## Build with WASM JIT support
	cargo build --features wasm-jit

build-distributed: ## Build with distributed features
	cargo build --features distributed

# Profiling and analysis
profile: ## Run with profiling enabled
	CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release
	@echo "Run with: perf record --call-graph=dwarf target/release/echo-repl"

bloat: ## Analyze binary size
	cargo bloat --release --crates

tree: ## Show dependency tree
	cargo tree

audit: ## Security audit of dependencies
	cargo audit