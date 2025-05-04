# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# FeatherFlow Development Guide

## Build Commands
- `make build` - Build the project
- `make run` - Run the application
- `make clean` - Clean build artifacts

## Test Commands
- `make test` - Run all tests
- `make test-module MODULE=<module>` - Run tests for specific module (e.g. `make test-module MODULE=sql_engine::ast_utils`)
- `make test-single TEST=<test>` - Run a specific test (e.g. `make test-single TEST=test_simple_select`)
- `make test-verbose` - Run tests with full output
- `make test-coverage` - Run tests with coverage report (HTML output)

## Lint Commands
- `make fmt` - Format code with rustfmt
- `make check-fmt` - Check formatting without modifying
- `make lint` - Run all linting (clippy + format checks)
- `make clippy` - Run clippy linter (warnings as errors)

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
***install with make ff-update first***
```
ff [COMMAND] [OPTIONS]

Available commands:
  parse    Parse SQL files and build a dependency graph
  version  Show version information
```

## Parse Command Options
```
ff parse [OPTIONS] --model-path <MODEL_PATH>

Options:
  -m, --model-path <MODEL_PATH>    Path to the SQL model files
  -f, --format <FORMAT>            Output format for the graph (dot, text, json, yaml) [default: text]
  -o, --output-file <OUTPUT_FILE>  File to write output to (if not provided, output to stdout)
```

## Testing with Demo Project
The demo project can be used to test FeatherFlow functionality:

- `make parse_demo_project` - Runs the parser on the demo project and outputs a YAML file to demo_project/output.yml
  - This generates a complete model graph with dependencies, columns, and metadata

The output YAML can be used for testing or feeding into other tools to visualize model structure.