PROJECT_DIR := feather_flow

.PHONY: all build fmt lint clippy test test-module test-single test-verbose run parse-example parse-dot parse-json clean help ci-test release install-target target target-release target-aarch64-linux prepare-binary

all: build ## Default target, builds the project

build: ## Build the project
	@echo "Building project..."
	@cd $(PROJECT_DIR) && cargo build

fmt: ## Format the code with rustfmt
	@echo "Formatting code..."
	@cd $(PROJECT_DIR) && cargo fmt

check-fmt: ## Check code formatting without modifying files
	@echo "Checking code formatting..."
	@cd $(PROJECT_DIR) && cargo fmt -- --check

lint: clippy check-fmt ## Run all linting checks (clippy + formatting)
	@echo "Linting completed"

clippy: ## Run clippy linter (warnings are errors)
	@echo "Running clippy..."
	@cd $(PROJECT_DIR) && cargo clippy -- -D warnings
	@cd $(PROJECT_DIR) && cargo clippy -- -D warnings
run: build ## Show how to run the application
	@echo "CLI built successfully. To run commands:"
	@echo "  cd $(PROJECT_DIR) && cargo run -- parse --model-path [PATH]"
	@echo "  Example: cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models"

parse-example: build ## Run parser on example models
	@echo "Running parser on example models..."
	@cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models

parse-dot: build ## Generate DOT graph from example models
	@echo "Generating DOT graph of example models..."
	@cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models --format dot > models.dot
	@echo "Graph saved to $(PROJECT_DIR)/models.dot"

parse-json: build ## Generate JSON representation from example models
	@echo "Generating JSON representation of example models..."
	@cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models --format json > models.json
	@echo "JSON saved to $(PROJECT_DIR)/models.json"


rust-folder-structure: ## Check project structure and dependencies
	cd $(PROJECT_DIR) && cargo check

install-target: ## Install a specific Rust target (usage: make install-target TARGET=<platform>)
	@echo "Installing target $(TARGET)..."
	@rustup target add $(TARGET) || true

target: rust-folder-structure install-target ## Build for specific target platform (usage: make target TARGET=<platform>)
	@echo "Building for target platform..."
	@cd $(PROJECT_DIR) && cargo build --target $(TARGET)

target-release: rust-folder-structure install-target ## Build release for specific target platform (usage: make target-release TARGET=<platform>)
	@echo "Building release for target platform..."
	@cd $(PROJECT_DIR) && cargo build --release --target $(TARGET)

target-aarch64-linux: rust-folder-structure ## Build release specifically for aarch64-linux-gnu
	@echo "Installing aarch64-unknown-linux-gnu target..."
	@rustup target add aarch64-unknown-linux-gnu || true
	@echo "Building aarch64-linux release with cross..."
	@cd $(PROJECT_DIR) && CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --release --target aarch64-unknown-linux-gnu

test: ## Run all tests
	@echo "Running tests..."
	@cd $(PROJECT_DIR) && cargo test

test-module: ## Run tests in specific module (usage: make test-module MODULE=commands::parse)
	@echo "Running tests in module $(MODULE)..."
	@cd $(PROJECT_DIR) && cargo test $(MODULE)

test-single: ## Run a specific test (usage: make test-single TEST=test_simple_select)
	@echo "Running test $(TEST)..."
	@cd $(PROJECT_DIR) && cargo test $(TEST)

test-verbose: ## Run tests with output (even for passing tests)
	@echo "Running tests with output..."
	@cd $(PROJECT_DIR) && cargo test -- --nocapture

test-coverage: ## Run tests with coverage report
	@echo "Running tests with coverage..."
	@cd $(PROJECT_DIR) && cargo tarpaulin --out Html

clean: ## Clean build artifacts
	@echo "Cleaning project..."
	@cd $(PROJECT_DIR) && cargo clean

prepare-binary: ## Prepare binary artifact for release (CI use)
	@echo "Preparing binary artifact..."
	@echo "Target: $(TARGET)"
	@echo "Binary name: $(BINARY_NAME)"
	@echo "Asset name: $(ASSET_NAME)"
	@mkdir -p artifacts
	@ls -la "$(PROJECT_DIR)/target/$(TARGET)/release/" || echo "Release directory does not exist"
	@if [ -f "$(PROJECT_DIR)/target/$(TARGET)/release/$(BINARY_NAME)" ]; then \
		cp "$(PROJECT_DIR)/target/$(TARGET)/release/$(BINARY_NAME)" "artifacts/$(ASSET_NAME)"; \
		echo "Binary copied successfully to artifacts/$(ASSET_NAME)"; \
		ls -la artifacts/; \
	else \
		echo "ERROR: Binary file not found at $(PROJECT_DIR)/target/$(TARGET)/release/$(BINARY_NAME)"; \
		exit 1; \
	fi

ci-test: ## Run tests using absolute paths (for CI environments)
	@echo "Running tests with absolute paths for CI environment..."
	@cargo test --manifest-path=$(CURDIR)/$(PROJECT_DIR)/Cargo.toml

release: ## Create a release tag (v0.1.0) and prepare for release
	@echo "Creating release v0.1.0..."
	@git tag -d v0.1.0 2>/dev/null || true
	@git tag v0.1.0
	@echo "Tagged v0.1.0 locally. To create a release:"
	@echo "  1. Push the tag: git push origin v0.1.0"
	@echo "  2. The GitHub Actions workflow will automatically build and publish the release"

help: ## Display this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
