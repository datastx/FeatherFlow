# FeatherFlow

## Project Vision

FeatherFlow is a Rust-based alternative to [dbt (data build tool)](https://www.getdbt.com/) designed to provide robust, statically-checked data transformations with superior performance. This project aims to solve common pain points in the data transformation workflow through static analysis and compiled performance.

## Installation

### Pre-built Binaries

You can download the latest pre-built binaries from the [GitHub Releases page](https://github.com/datastx/FeatherFlow/releases).

#### Linux
```bash
# For x86_64 systems
curl -L -o featherflow https://github.com/datastx/FeatherFlow/releases/latest/download/featherflow-linux-amd64
chmod +x featherflow
sudo mv featherflow /usr/local/bin/

# For ARM64 systems
curl -L -o featherflow https://github.com/datastx/FeatherFlow/releases/latest/download/featherflow-linux-arm64
chmod +x featherflow
sudo mv featherflow /usr/local/bin/
```

#### macOS
```bash
# For Intel Macs
curl -L -o featherflow https://github.com/datastx/FeatherFlow/releases/latest/download/featherflow-macos-amd64
chmod +x featherflow
sudo mv featherflow /usr/local/bin/

# For Apple Silicon (M1/M2)
curl -L -o featherflow https://github.com/datastx/FeatherFlow/releases/latest/download/featherflow-macos-arm64
chmod +x featherflow
sudo mv featherflow /usr/local/bin/
```

#### Windows
Download the [Windows binary](https://github.com/datastx/FeatherFlow/releases/latest/download/featherflow-windows-amd64.exe) and add it to your PATH.

### Why FeatherFlow?

**Problems with existing tools like dbt:**
- Runtime SQL errors that could be caught earlier with static analysis
- Performance limitations of Python-based tooling
- Limited capabilities for deep SQL introspection and optimization

**FeatherFlow's approach:**
- Parse SQL into Abstract Syntax Trees (ASTs) to perform static analysis
- Catch schema issues, syntax errors, and semantic problems before execution
- Compile transformation pipelines to efficient execution plans
- Provide a robust, type-safe experience for data engineers

### Key Features

- **Static SQL Analysis**: Catch errors before runtime by analyzing the SQL AST
- **Schema Validation**: Validate schema references without connecting to a database
- **Dependency Management**: Automatically track dependencies between models
- **Performance**: Rust-based implementation for superior speed and memory efficiency
- **SQL Transformation**: Intelligent handling of SQL queries with schema manipulation

### Current Development Status

FeatherFlow is in active development. Current focus areas include:
- SQL parsing and AST manipulation
- Schema transformation and validation
- Building core transformation engine components

# Development

## Builds and Releases

### Continuous Builds

Every push to the `main` or `master` branch automatically:
- Builds binaries for all supported platforms (Linux, macOS, Windows)
- Archives the binaries as GitHub Actions artifacts for 14 days
- No GitHub release is created for these builds

To access these builds, go to the GitHub Actions tab, select the most recent workflow run, and download the `featherflow-all-binaries` artifact.

### Creating a Release

To create an official release with prebuilt binaries:

1. Use the make command to create a release tag:
   ```bash
   make release
   git push origin v0.1.0
   ```

2. Alternatively, manually trigger the release workflow from the GitHub Actions tab, providing the version tag.

The GitHub Actions workflow will automatically:
- Build binaries for Linux (x86_64, ARM64), macOS (x86_64, ARM64), and Windows
- Create a GitHub Release with these binaries
- Generate release notes based on the commits since the last release

## Running Tests

You can run tests using the following commands:

```bash
# Run all tests in the project
make test

# Run only the sql_engine/ast_utils tests
make test-module MODULE=sql_engine::ast_utils

# Run a specific test
make test-single TEST=test_simple_select

# Run tests with output (even for passing tests)
make test-verbose
```

## Test Structure
For tests to be discovered by Cargo:
1. Unit tests should be in the same file as the code they test, in a `mod tests` module
2. Integration tests should be directly in the `tests/` directory (not in `tests/src/`)

If your tests aren't running, ensure they follow this structure.