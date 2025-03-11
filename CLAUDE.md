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

## Lint Commands
- `make fmt` - Format code with rustfmt
- `make lint` - Lint code with clippy (warnings as errors)

## Code Style Guidelines
- **Naming**: Use snake_case for functions/variables, CamelCase for types/traits, UPPERCASE for constants
- **Imports**: Group by: 1) std lib, 2) external crates, 3) local imports
- **Error Handling**: Return Result types in library code; propagate with `?`
- **Documentation**: Use doc comments (`///`) for public APIs, module docs with `//!`
- **Testing**: Unit tests in `mod tests` within files; integration tests in `tests/` directory

## CLI Usage
```
featherflow [COMMAND] [OPTIONS]
```