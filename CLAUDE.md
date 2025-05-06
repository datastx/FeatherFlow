# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# FeatherFlow Development Guide

## Project Structure
- `feather_flow/` - Main Rust project directory
  - `src/` - Source code
    - `commands/` - CLI command implementations (parse, validate)
    - `sql_engine/` - Core SQL parsing and analysis functionality
      - `ast_utils.rs` - AST manipulation utilities
      - `extractors.rs` - Extract information from SQL AST
      - `lineage.rs` - Data lineage analysis
      - `sql_model.rs` - SQL model representation
      - `tables.rs` - Table handling utilities
    - `validators/` - Model validation functionality
    - `feather_lang/` - Custom language components
      - `lexer/` - Lexical analysis
      - `token/` - Token definitions
      - `repl/` - Read-Eval-Print Loop implementation
  - `Cargo.toml` - Rust project manifest and dependencies
- `demo_project/` - Example project for testing

## Makefile Tasks

### Build and Installation
- `make build` - Build the project
- `make run` - Run the application
- `make clean` - Clean build artifacts
- `make ff-local` - Build the ff CLI in release mode
- `make install-local` - Install ff CLI locally (to ~/.local/bin)
- `make ff-update` - Update ff CLI to the latest local version

### Cross-Platform and Target-Specific Building
- `make target TARGET=<platform>` - Build for a specific target platform
- `make target-release TARGET=<platform>` - Build release for specific target platform
- `make target-aarch64-linux` - Build release specifically for aarch64-linux-gnu
- `make install-target TARGET=<platform>` - Install a specific Rust target

### Test Commands
- `make test` - Run all tests
- `make test-module MODULE=<module>` - Run tests for specific module (e.g. `make test-module MODULE=sql_engine::ast_utils`)
- `make test-single TEST=<test>` - Run a specific test (e.g. `make test-single TEST=test_simple_select`)
- `make test-verbose` - Run tests with full output
- `make test-coverage` - Run tests with coverage report (HTML output)
- `make ci-test` - Run tests using absolute paths (for CI environments)

### Lint and Format Commands
- `make fmt` - Format code with rustfmt
- `make check-fmt` - Check formatting without modifying
- `make lint` - Run all linting (clippy + format checks)
- `make clippy` - Run clippy linter (warnings as errors)

### Version Management
- `make version` - Check installed ff version
- `make current-version` - Display current version from Cargo.toml
- `make bump-version NEW_VERSION=x.y.z` - Update version in Cargo.toml
- `make new-version NEW_VERSION=x.y.z` - Bump version, build, and prepare for release
- `make release` - Create a release tag based on current version

### Parser and Demo Project
- `make parse-example` - Run parser on example models
- `make parse-dot` - Generate DOT graph from example models
- `make parse-json` - Generate JSON representation from example models
- `make parse_demo_project` - Run parser on demo project and output YAML

### Git Utilities
- `make clean-branches` - Clean up old branches

## Dependencies (from Cargo.toml)
- **Core**: anyhow, clap, colored, petgraph
- **SQL Parsing**: sqlparser
- **File Handling**: walkdir
- **Data Processing**: chrono, rand, csv, serde, serde_json, serde_yaml, sha2
- **Testing**: tempfile, criterion, pretty_assertions, test-case

## Code Style Guidelines
- **Naming**: Use snake_case for functions/variables, CamelCase for types/traits, UPPERCASE for constants
- **Imports**: Group by: 1) std lib, 2) external crates, 3) local imports
- **Error Handling**: Return Result types; propagate with `?`; provide clear error messages
- **Documentation**: Use doc comments (`///`) for public APIs, module docs with `//!`
- **Formatting**: Max width 100 chars, 4 spaces for indentation, Unix newlines
- **Testing**: Unit tests in `mod tests` within files; integration tests in `tests/` directory
- **Function Design**: Keep functions small and focused on a single task
- **SQL Engine**: Parse SQL to AST, manipulate AST, convert back to SQL text

## CLI Usage
***Install with `make ff-update` first***
```
ff [COMMAND] [OPTIONS]

Available commands:
  parse     Parse SQL files and build a dependency graph
  validate  Validate model file structure
  version   Show version information
```

## Parse Command Options
```
ff parse [OPTIONS] --model-path <MODEL_PATH>

Options:
  -m, --model-path <MODEL_PATH>    Path to the SQL model files
  -f, --format <FORMAT>            Output format for the graph (dot, text, json, yaml) [default: text]
  -o, --output-file <OUTPUT_FILE>  File to write output to (if not provided, output to stdout)
```

## Validate Command Options
```
ff validate [OPTIONS] --model-path <MODEL_PATH>

Options:
  -m, --model-path <MODEL_PATH>    Path to the SQL model files
  -q, --quiet                      Quiet mode - only output errors
```

## Testing with Demo Project
The demo project can be used to test FeatherFlow functionality:

- `make parse_demo_project` - Runs the parser on the demo project and outputs a YAML file to demo_project/output.yml
  - This generates a complete model graph with dependencies, columns, and metadata

The output YAML can be used for testing or feeding into other tools to visualize model structure.