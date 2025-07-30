.PHONY: all build test clean release install dev-setup help
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
NPM := npm
PYTHON := python3
DOCKER := docker
PRE_COMMIT := pre-commit

# Rust build profiles
PROFILE ?= debug
RELEASE_FLAGS := $(if $(filter release,$(PROFILE)),--release,)

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

##@ General

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make ${CYAN}<target>${NC}\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  ${CYAN}%-15s${NC} %s\n", $$1, $$2 } /^##@/ { printf "\n${YELLOW}%s${NC}\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Development

dev-setup: ## Set up development environment
	@echo "${GREEN}Setting up development environment...${NC}"
	@command -v rustup >/dev/null 2>&1 || { echo "${RED}rustup not found. Please install Rust.${NC}"; exit 1; }
	@rustup component add rustfmt clippy rust-analyzer
	@command -v cargo-tarpaulin >/dev/null 2>&1 || cargo install cargo-tarpaulin
	@command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit
	@command -v cargo-outdated >/dev/null 2>&1 || cargo install cargo-outdated
	@command -v cargo-edit >/dev/null 2>&1 || cargo install cargo-edit
	@echo "${GREEN}Installing Node.js dependencies...${NC}"
	@$(NPM) install
	@echo "${GREEN}Installing pre-commit hooks...${NC}"
	@pip install pre-commit
	@$(PRE_COMMIT) install
	@echo "${GREEN}Development environment setup complete!${NC}"

install-hooks: ## Install git pre-commit hooks
	@$(PRE_COMMIT) install
	@$(PRE_COMMIT) install --hook-type commit-msg
	@$(PRE_COMMIT) install --hook-type pre-push
	@echo "${GREEN}Pre-commit hooks installed${NC}"

##@ Building

all: build test ## Build and test everything

build: ## Build all workspace crates
	@echo "${GREEN}Building workspace...${NC}"
	@$(CARGO) build --workspace --all-features $(RELEASE_FLAGS)

build-release: ## Build all crates in release mode
	@$(MAKE) build PROFILE=release

tree-sitter: ## Generate tree-sitter parser
	@echo "${GREEN}Generating tree-sitter parser...${NC}"
	@npx tree-sitter generate
	@npx tree-sitter test

wasm: ## Build WASM targets
	@echo "${GREEN}Building WASM targets...${NC}"
	@$(CARGO) build --target wasm32-unknown-unknown --package echo-core
	@npx tree-sitter build --wasm

##@ Testing

test: ## Run all tests
	@echo "${GREEN}Running tests...${NC}"
	@$(CARGO) test --workspace --all-features

test-unit: ## Run unit tests only
	@$(CARGO) test --workspace --all-features --lib

test-integration: ## Run integration tests
	@$(CARGO) test --workspace --all-features --tests

test-doc: ## Run documentation tests
	@$(CARGO) test --workspace --all-features --doc

test-echo: ## Run Echo language test suite
	@echo "${GREEN}Running Echo test suite...${NC}"
	@cd echo-repl && ./test.sh

test-tree-sitter: ## Run tree-sitter tests
	@npx tree-sitter test

coverage: ## Generate code coverage report
	@echo "${GREEN}Generating coverage report...${NC}"
	@$(CARGO) tarpaulin --verbose --all-features --workspace \
		--timeout 300 --out html --out lcov \
		--exclude-files "*/tests/*" --exclude-files "*/examples/*"
	@echo "${GREEN}Coverage report generated at tarpaulin-report.html${NC}"

bench: ## Run benchmarks
	@echo "${GREEN}Running benchmarks...${NC}"
	@$(CARGO) bench --workspace --all-features

##@ Code Quality

fmt: ## Format all code
	@echo "${GREEN}Formatting code...${NC}"
	@$(CARGO) fmt --all
	@npx prettier --write "**/*.{json,yml,yaml,md}" --ignore-path .prettierignore

fmt-check: ## Check code formatting
	@$(CARGO) fmt --all -- --check
	@npx prettier --check "**/*.{json,yml,yaml,md}" --ignore-path .prettierignore

lint: ## Run all linters
	@echo "${GREEN}Running linters...${NC}"
	@$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings
	@npx eslint . --ext .js,.ts || true
	@$(PYTHON) -m ruff check echo-repl/run_echo_tests.py || true

fix: ## Auto-fix linting issues
	@$(CARGO) clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
	@npx eslint . --ext .js,.ts --fix || true
	@$(PYTHON) -m ruff check --fix echo-repl/run_echo_tests.py || true

audit: ## Run security audit
	@echo "${GREEN}Running security audit...${NC}"
	@$(CARGO) audit

deps-update: ## Check for outdated dependencies
	@echo "${GREEN}Checking for outdated dependencies...${NC}"
	@$(CARGO) outdated -R
	@$(NPM) outdated || true

pre-commit: ## Run pre-commit hooks on all files
	@$(PRE_COMMIT) run --all-files

##@ Documentation

docs: ## Generate and open documentation
	@echo "${GREEN}Generating documentation...${NC}"
	@$(CARGO) doc --workspace --all-features --no-deps --open

docs-build: ## Build documentation without opening
	@$(CARGO) doc --workspace --all-features --no-deps

##@ Running

run-repl: ## Run the Echo REPL
	@$(CARGO) run --package echo-repl

run-web: ## Run the Echo web server
	@$(CARGO) run --package echo-web

run-example: ## Run an example (use EXAMPLE=name)
	@$(CARGO) run --example $(EXAMPLE)

##@ Docker

docker-build: ## Build Docker image
	@echo "${GREEN}Building Docker image...${NC}"
	@$(DOCKER) build -t moor-echo:latest .

docker-run: ## Run Docker container
	@$(DOCKER) run -it --rm -p 3000:3000 moor-echo:latest

docker-compose: ## Run with docker-compose
	@$(DOCKER) compose up

##@ Release

release-dry: ## Perform a dry run of the release process
	@echo "${GREEN}Performing release dry run...${NC}"
	@$(CARGO) publish --dry-run --package echo-core
	@$(CARGO) publish --dry-run --package echo-repl
	@$(CARGO) publish --dry-run --package echo-web

release-patch: ## Release a patch version
	@$(CARGO) release patch --workspace --execute

release-minor: ## Release a minor version
	@$(CARGO) release minor --workspace --execute

release-major: ## Release a major version
	@$(CARGO) release major --workspace --execute

##@ Maintenance

clean: ## Clean all build artifacts
	@echo "${GREEN}Cleaning build artifacts...${NC}"
	@$(CARGO) clean
	@rm -rf target/
	@rm -rf node_modules/
	@rm -rf dist/
	@rm -f tarpaulin-report.html cobertura.xml
	@find . -name "*.pyc" -delete
	@find . -name "__pycache__" -type d -exec rm -rf {} +

reset: ## Reset to clean state (removes all generated files)
	@$(MAKE) clean
	@rm -rf Cargo.lock package-lock.json
	@git clean -fdx -e .env -e .env.local

ci-local: ## Run CI checks locally
	@echo "${GREEN}Running CI checks locally...${NC}"
	@$(MAKE) fmt-check
	@$(MAKE) lint
	@$(MAKE) test
	@$(MAKE) audit
	@echo "${GREEN}All CI checks passed!${NC}"

##@ Utilities

watch: ## Watch for changes and rebuild
	@$(CARGO) watch -x "build --workspace --all-features"

watch-test: ## Watch for changes and run tests
	@$(CARGO) watch -x "test --workspace --all-features"

todo: ## List all TODO/FIXME comments
	@echo "${GREEN}TODO/FIXME items:${NC}"
	@grep -rn "TODO\|FIXME" --include="*.rs" --include="*.js" --include="*.ts" --include="*.py" src/ || echo "No TODOs found!"

loc: ## Count lines of code
	@echo "${GREEN}Lines of code:${NC}"
	@tokei . -e target -e node_modules -e dist || cloc . --exclude-dir=target,node_modules,dist || echo "Install tokei or cloc for line counting"

check-links: ## Check for broken links in documentation
	@echo "${GREEN}Checking links in documentation...${NC}"
	@command -v markdown-link-check >/dev/null 2>&1 || npm install -g markdown-link-check
	@find . -name "*.md" -not -path "./target/*" -not -path "./node_modules/*" | xargs -I {} markdown-link-check {}