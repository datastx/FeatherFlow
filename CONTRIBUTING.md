# Contributing to FeatherFlow

Thank you for your interest in contributing to FeatherFlow! As a Rust-based alternative to dbt focusing on static analysis, we welcome contributions that help advance our mission of providing robust, performant data transformations.

## Development Setup

1. **Requirements**:
   - Rust toolchain (latest stable recommended)
   - Cargo
   - Git

2. **Clone the repository**:
   ```bash
   git clone https://github.com/datastx/FeatherFlow.git
   cd FeatherFlow
   ```

3. **Build the project**:
   ```bash
   make build
   ```

4. **Run tests**:
   ```bash
   make test
   ```

## Project Structure

- `feather_flow/src/`: Core implementation
  - `bin/`: Executable entry points
  - `commands/`: CLI implementation
  - `feather_lang/`: Domain-specific language implementation
  - `sql_engine/`: SQL parsing and transformation
- `feather_flow/tests/`: Integration tests
- `Makefile`: Build and test commands

## Development Workflow

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Follow Rust coding conventions
   - Add tests for new functionality
   - Update documentation as needed

3. **Run tests**:
   ```bash
   make test
   ```

4. **Format code**:
   ```bash
   make fmt
   ```

5. **Run linter**:
   ```bash
   make lint
   ```

6. **Create a pull request**:
   - Use a clear title and description
   - Reference any related issues
   - Fill out the PR template

## Development Guidelines

### Code Style

- Follow Rust idioms and conventions
- Use descriptive variable names
- Document public APIs with rustdoc comments
- Keep functions small and focused on a single task

### Testing

- Add unit tests for new functionality
- Add integration tests for end-to-end behavior
- Test edge cases and error conditions
- Use test fixtures for complex test scenarios

### SQL Engine Development

When working on the SQL engine components:

1. Parse SQL into ASTs using sqlparser with appropriate dialect
2. Manipulate the AST for transformations
3. Convert the modified AST back to SQL text
4. Add tests with a variety of SQL statements to ensure correct behavior

### Feature Development

When adding new features:

1. Discuss the feature in an issue first
2. Consider how it fits into the overall architecture
3. Plan for backward compatibility
4. Document the new functionality
5. Add comprehensive tests

## Pull Request Process

1. Update the README.md or documentation with details of changes if appropriate
2. Update the CHANGELOG.md with a description of your changes
3. The PR will be reviewed by maintainers
4. Once approved, the PR will be merged

## Getting Help

If you have questions or need help, you can:

- Open an issue with your question
- Reach out to the maintainers

Thank you for contributing to FeatherFlow!