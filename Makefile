PROJECT_DIR := feather_flow

.PHONY: all build fmt lint test test-module test-single test-verbose run clean help target fix-deps ci-test

all: build


build:
	@echo "Building project..."
	@cd $(PROJECT_DIR) && cargo build


fmt:
	@echo "Formatting code..."
	@cd $(PROJECT_DIR) && cargo fmt


lint:
	@echo "Linting code..."
	@cd $(PROJECT_DIR) && cargo clippy -- -D warnings

cli: build
	@echo "CLI built successfully. To run commands:"
	@echo "  cd $(PROJECT_DIR) && cargo run --bin featherflow -- [COMMAND] [OPTIONS]"
	@echo "  Example: cd $(PROJECT_DIR) && cargo run --bin featherflow -- help 

run-cli: build
	@echo "Running example CLI commands..."
	@cd $(PROJECT_DIR) && cargo run --bin featherflow -- help

rust-folder-structure:
	cd $(PROJECT_DIR) && cargo check

target: rust-folder-structure
	@echo "Building for target platform..."
	@cd $(PROJECT_DIR) && cargo build --target $(TARGET)

test:
	@echo "Running tests..."
	@cd $(PROJECT_DIR) && cargo test

test-module:
	@echo "Running tests in module $(MODULE)..."
	@cd $(PROJECT_DIR) && cargo test $(MODULE)

test-single:
	@echo "Running test $(TEST)..."
	@cd $(PROJECT_DIR) && cargo test $(TEST)

test-verbose:
	@echo "Running tests with output..."
	@cd $(PROJECT_DIR) && cargo test -- --nocapture

run:
	@echo "Running application..."
	@cd $(PROJECT_DIR) && cargo run --bin feather_flow

clean:
	@echo "Cleaning project..."
	@cd $(PROJECT_DIR) && cargo clean

ci-test:
	@echo "Running tests with absolute paths for CI environment..."
	@cargo test --manifest-path=$(CURDIR)/$(PROJECT_DIR)/Cargo.toml

help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  fmt          - Format the code with rustfmt"
	@echo "  lint         - Lint the code with cargo clippy (warnings are errors)"
	@echo "  test         - Run all tests"
	@echo "  test-module  - Run tests in specific module (usage: make test-module MODULE=sql_engine::ast_utils)"
	@echo "  test-single  - Run a specific test (usage: make test-single TEST=test_simple_select)"
	@echo "  test-verbose - Run tests with output (even for passing tests)"
	@echo "  run          - Run the application"
	@echo "  clean        - Clean build artifacts"
	@echo "  target       - Build for specific target platform (usage: make target TARGET=<platform>)"
	@echo "  fix-deps     - Fix dependency conflicts by updating and adding overrides"
	@echo "  ci-test      - Run tests using absolute paths (for CI environments)"
