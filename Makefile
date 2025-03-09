PROJECT_DIR := feather_flow

.PHONY: all build fmt lint test run clean help target fix-deps

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
	@echo "  Example: cd $(PROJECT_DIR) && cargo run --bin featherflow -- start --file workflow.yaml --name test-workflow"

run-cli: build
	@echo "Running example CLI commands..."
	@cd $(PROJECT_DIR) && cargo run --bin featherflow -- start --file workflow.yaml --name test-workflow
	@cd $(PROJECT_DIR) && cargo run --bin featherflow -- list --status running

target:
	@echo "Building for target platform..."
	@cd $(PROJECT_DIR) && cargo build --target $(TARGET)

test:
	@echo "Running tests..."
	@cd $(PROJECT_DIR) && cargo test

run:
	@echo "Running application..."
	@cd $(PROJECT_DIR) && cargo run

clean:
	@echo "Cleaning project..."
	@cd $(PROJECT_DIR) && cargo clean

help:
	@echo "Available targets:"
	@echo "  build    - Build the project"
	@echo "  fmt      - Format the code with rustfmt"
	@echo "  lint     - Lint the code with cargo clippy (warnings are errors)"
	@echo "  test     - Run tests"
	@echo "  run      - Run the application"
	@echo "  clean    - Clean build artifacts"
	@echo "  target   - Build for specific target platform (usage: make target TARGET=<platform>)"
	@echo "  fix-deps - Fix dependency conflicts by updating and adding overrides"
