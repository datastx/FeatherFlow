# FeatherFlow Architecture

## Overview

FeatherFlow is architected to provide a robust, statically-checked alternative to dbt (data build tool) with superior performance through Rust implementation. This document outlines the core architectural components and their interactions.

## Core Components

### 1. SQL Engine

The SQL engine is responsible for parsing, analyzing, and transforming SQL statements:

- **SQL Parsing**: Uses the sqlparser-rs crate to parse SQL into Abstract Syntax Trees (ASTs)
- **AST Manipulation**: Transforms SQL by manipulating the AST (e.g., modifying schema references)
- **Static Analysis**: Analyzes SQL for potential errors without executing it
- **Dependency Extraction**: Identifies tables and columns referenced in queries to build dependency graphs
- **Column Lineage Tracking**: Tracks data flow from source to target columns through transformations

### 2. Feather Language

A domain-specific language for configuring and extending the capabilities of FeatherFlow:

- **Lexer/Parser**: Tokenizes and parses the Feather language
- **REPL**: Provides an interactive environment for testing Feather language scripts
- **Configuration**: Allows configuring transformation pipelines declaratively

### 3. CLI Tools

Command-line tools for interacting with FeatherFlow:

- **Project Initialization**: Setting up new FeatherFlow projects
- **Transformation Execution**: Running transformations and pipelines
- **Validation**: Running static analysis on project SQL files
- **Documentation Generation**: Creating docs from project metadata
- **Lineage Visualization**: Generating column-level lineage graphs

## Data Flow

1. **Input**: SQL files, Feather language configuration
2. **Parsing**: Conversion to ASTs and object models
3. **Analysis**: Static checking, dependency resolution, lineage extraction
4. **Transformation**: Schema manipulation, optimization
5. **Execution**: (Optional) Running against a database
6. **Documentation**: Generation of data lineage and docs

## Technical Design Decisions

### Why Rust?

- **Performance**: Compiled language with zero-cost abstractions
- **Memory Safety**: Prevents common memory issues without garbage collection
- **Concurrency**: Safe concurrency with the ownership model
- **Tooling**: Strong type system and compiler catches errors early

### AST-Based Approach

By parsing SQL into Abstract Syntax Trees before execution, FeatherFlow can:

1. Detect errors statically (before runtime)
2. Understand complex relationships between tables and columns
3. Perform optimizations and transformations at the query level
4. Generate documentation and lineage information automatically
5. Track column-level data flow through transformations

### Project Structure

```
feather_flow/
├── src/
│   ├── bin/               # Command-line executables
│   ├── commands/          # CLI command implementations
│   ├── feather_lang/      # The Feather domain-specific language
│   │   ├── lexer/         # Tokenization
│   │   ├── repl/          # Interactive shell
│   │   └── token/         # Token definitions
│   ├── sql_engine/        # SQL parsing and manipulation
│   │   ├── ast_utils.rs   # AST transformation utilities
│   │   ├── lineage.rs     # Column-level lineage tracking
│   │   ├── tables.rs      # Table metadata management
│   │   └── ...
│   └── lib.rs             # Library exports
├── tests/                 # Integration tests
└── ...
```

## Column-Level Lineage

FeatherFlow implements sophisticated column-level lineage tracking:

1. **Source Extraction**: Identifies source columns in a query
2. **Transformation Classification**: Categorizes transformations (direct, aggregation, expression, etc.)
3. **Target Mapping**: Maps source columns to target columns in the result set
4. **Visualization**: Generates DOT format graphs (compatible with Graphviz) for lineage visualization
5. **Metadata**: Stores lineage metadata for documentation and analysis

This enables:
- Understanding data flow throughout the transformation pipeline
- Impact analysis for schema changes
- Documentation of data transformations
- Quality and governance enforcement

## Future Directions

1. **Schema Inference**: Automatically infer and validate schemas
2. **Incremental Building**: Smart rebuilding of only affected models
3. **Advanced Optimizations**: Query rewriting for performance
4. **Cross-Database Support**: Abstract over different SQL dialects
5. **Integration with Data Catalogs**: Connect with external metadata systems
6. **Improved Lineage**: Handle more complex transformations (UDFs, window functions, CTEs)

## Contributing

When contributing to FeatherFlow, keep these architectural principles in mind:

1. Favor static analysis over runtime checking
2. Maintain a clear separation between parsing, analysis, and execution
3. Design for extensibility to support different databases and use cases
4. Prioritize user experience and clear error messaging