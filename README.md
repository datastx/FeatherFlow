# FeatherFlow

### High level overview
- A simple blazingly fast transformation framework for large scale data processing.

# Running Tests

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