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

## Why FeatherFlow?

**Problems with existing tools like dbt:**
- Runtime SQL errors that could be caught earlier with static analysis
- Performance limitations of Python-based tooling
- Limited capabilities for deep SQL introspection and optimization

**FeatherFlow's approach:**
- Parse SQL into Abstract Syntax Trees (ASTs) to perform static analysis
- Catch schema issues, syntax errors, and semantic problems before execution
- Compile transformation pipelines to efficient execution plans
- Provide a robust, type-safe experience for data engineers

## Core Components

### 1. SQL Parser

The SQL parser is responsible for analyzing SQL statements to extract dependencies:

- **SQL Parsing**: Uses the sqlparser-rs crate to parse SQL into Abstract Syntax Trees (ASTs)
- **Dependency Extraction**: Identifies tables referenced in queries to build dependency graphs

### 2. Graph Builder

Handles the construction and analysis of dependency graphs:

- **DAG Construction**: Builds directed acyclic graphs from model dependencies
- **Cycle Detection**: Identifies circular dependencies in the model graph
- **Visualization**: Generates representations of the graph in various formats

### 3. CLI Interface

A simple command-line interface for interacting with FeatherFlow:

- **Parse Command**: Process SQL files in a specified directory
- **Output Options**: Control the format of the generated dependency information

## Key Features

- **SQL Dependency Analysis**: Extract and visualize dependencies between SQL models
- **Cycle Detection**: Identify circular references in your data models
- **Multiple Output Formats**: Generate dependency information as text, DOT graphs, or JSON
- **Performance**: Rust-based implementation for superior speed and memory efficiency
- **Static Analysis**: Parse SQL without executing it to detect potential issues early

## Current Development Status

FeatherFlow v0.1 focuses on a minimal viable implementation with these capabilities:
- Parsing SQL files to extract table dependencies
- Building dependency graphs (DAGs) between models
- Detecting circular dependencies
- Outputting dependency information in various formats (text, DOT, JSON)

## Usage Examples

```bash
# Parse SQL files in the models directory and show text output
ff parse --model-path ./models

# Generate DOT format for visualization
ff parse --model-path ./models --format dot > models.dot
dot -Tpng -o models.png models.dot

# Output as JSON for further processing
ff parse --model-path ./models --format json > models.json
```

You can also use the Makefile to run these examples:

```bash
# Run parser on example models with text output
make parse-example

# Generate DOT graph from example models
make parse-dot

# Generate JSON representation from example models
make parse-json
```

# Development

## Building the Project

```bash
# Build the project
make build

# Run linting checks
make lint

# Format code
make fmt
```

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

## Cross-Platform Builds

You can build for specific target platforms using:

```bash
# Build for a specific target platform
make target TARGET=<platform>

# Build release version for a specific target platform
make target-release TARGET=<platform>

# Example: Build for ARM64 Linux
make target-release TARGET=aarch64-unknown-linux-gnu
```

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

# Run tests with coverage report
make test-coverage
```

## Test Structure
For tests to be discovered by Cargo:
1. Unit tests should be in the same file as the code they test, in a `mod tests` module
2. Integration tests should be directly in the `tests/` directory (not in `tests/src/`)

If your tests aren't running, ensure they follow this structure.

## Additional Make Targets

Run `make help` to see all available make targets with descriptions.