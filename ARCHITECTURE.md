# FeatherFlow Architecture

## Overview

FeatherFlow is architected to provide a robust, statically-checked alternative to dbt (data build tool) with superior performance through Rust implementation. This document outlines the core architectural components and their interactions.

### Current Focus (v0.1)

The initial version of FeatherFlow (v0.1) focuses on a minimal viable implementation with these capabilities:
- Parsing SQL files to extract table dependencies
- Building dependency graphs (DAGs) between models
- Detecting circular dependencies
- Outputting dependency information in various formats (text, DOT, JSON)

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

## Data Flow

1. **Input**: SQL files in the specified model directory
2. **Parsing**: Conversion to ASTs to extract table references
3. **Graph Building**: Construction of dependency graph between models
4. **Analysis**: Cycle detection and dependency resolution
5. **Output**: Generation of dependency information in the specified format

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
│   ├── main.rs            # Binary entry point with CLI handling
│   ├── sql_parser.rs      # SQL parsing functionality
│   └── graph.rs           # Graph building and visualization
├── Cargo.toml             # Dependencies
└── tests/                 # Integration tests
```
## Table Dependencies

FeatherFlow extracts table-level dependencies from SQL models:

1. **Reference Extraction**: Identifies tables referenced in FROM clauses and JOINs
2. **Model Mapping**: Maps referenced tables to their corresponding models
3. **Graph Construction**: Builds a directed graph with edges representing dependencies
4. **Visualization**: Generates DOT format graphs (compatible with Graphviz) for dependency visualization

This enables:
- Understanding dependencies between models
- Detecting circular references
- Planning execution order
- Impact analysis for model changes
- Quality and governance enforcement

## Future Directions

Once the initial version is complete, FeatherFlow can be extended with:

1. **Column-Level Lineage**: Track data flow at the column level through transformations
2. **Schema Transformation**: Modify schema references in SQL models
3. **Static Validation**: Analyze SQL for potential errors without execution
4. **Feather Language**: Add a domain-specific language for configuration
5. **Incremental Building**: Smart rebuilding of only affected models
6. **Cross-Database Support**: Abstract over different SQL dialects

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

## Contributing

When contributing to FeatherFlow, keep these architectural principles in mind:

1. Favor static analysis over runtime checking
2. Maintain a clear separation between parsing, analysis, and execution
3. Design for extensibility to support different databases and use cases
4. Prioritize user experience and clear error messaging
5. Start simple, then incrementally add complexity