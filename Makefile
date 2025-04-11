PROJECT_DIR := feather_flow

.PHONY: all build fmt lint clippy test test-module test-single test-verbose run parse-example parse-dot parse-json clean help ci-test release

all: build


build:
	@echo "Building project..."
	@cd $(PROJECT_DIR) && cargo build

fmt:
	@echo "Formatting code..."
	@cd $(PROJECT_DIR) && cargo fmt

check-fmt:
	@echo "Checking code formatting..."
	@cd $(PROJECT_DIR) && cargo fmt -- --check

lint: clippy check-fmt
	@echo "Linting completed"

clippy:
	@echo "Running clippy..."
	@cd $(PROJECT_DIR) && cargo clippy -- -D warnings
	@cd $(PROJECT_DIR) && cargo clippy -- -D warnings
run: build
	@echo "CLI built successfully. To run commands:"
	@echo "  cd $(PROJECT_DIR) && cargo run -- parse --model-path [PATH]"
	@echo "  Example: cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models"

parse-example: build
	@echo "Running parser on example models..."
	@cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models

parse-dot: build
	@echo "Generating DOT graph of example models..."
	@cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models --format dot > models.dot
	@echo "Graph saved to $(PROJECT_DIR)/models.dot"

parse-json: build
	@echo "Generating JSON representation of example models..."
	@cd $(PROJECT_DIR) && cargo run -- parse --model-path ./models --format json > models.json
	@echo "JSON saved to $(PROJECT_DIR)/models.json"


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

test-coverage:
	@echo "Running tests with coverage..."
	@cd $(PROJECT_DIR) && cargo tarpaulin --out Html

clean:
	@echo "Cleaning project..."
	@cd $(PROJECT_DIR) && cargo clean

ci-test:
	@echo "Running tests with absolute paths for CI environment..."
	@cargo test --manifest-path=$(CURDIR)/$(PROJECT_DIR)/Cargo.toml

release:
	@echo "Creating release v0.1.0..."
	@git tag -d v0.1.0 2>/dev/null || true
	@git tag v0.1.0
	@echo "Tagged v0.1.0 locally. To create a release:"
	@echo "  1. Push the tag: git push origin v0.1.0"
	@echo "  2. The GitHub Actions workflow will automatically build and publish the release"

help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  fmt          - Format the code with rustfmt"
	@echo "  check-fmt    - Check code formatting without modifying files"
	@echo "  lint         - Run all linting checks (clippy + formatting)"
	@echo "  clippy       - Run clippy linter (warnings are errors)"
	@echo "  test         - Run all tests"
	@echo "  test-module  - Run tests in specific module (usage: make test-module MODULE=commands::parse)"
	@echo "  test-single  - Run a specific test (usage: make test-single TEST=test_simple_select)"
	@echo "  test-verbose - Run tests with output (even for passing tests)"
	@echo "  test-coverage - Run tests with coverage report"
	@echo "  run          - Show how to run the application"
	@echo "  parse-example - Run parser on example models directory"
	@echo "  parse-dot    - Generate DOT graph from example models"
	@echo "  parse-json   - Generate JSON representation from example models"
	@echo "  clean        - Clean build artifacts"
	@echo "  target       - Build for specific target platform (usage: make target TARGET=<platform>)"
	@echo "  ci-test      - Run tests using absolute paths (for CI environments)"
	@echo "  release      - Create a release tag (v0.1.0) and prepare for release"
