# Demo Financial Dataset for FeatherFlow

.PHONY: all setup generate load transform clean visualize

# Default target
all: setup generate load transform

# Create necessary directories
setup:
	@mkdir -p demo_project/data
	@mkdir -p demo_project/models/staging
	@mkdir -p demo_project/models/marts/core
	@mkdir -p demo_project/models/marts/finance
	@mkdir -p demo_project/seeds

# Generate synthetic financial data
generate:
	@echo "Generating synthetic financial data..."
	@cargo run --bin ff -- demo generate

# Create DuckDB database and load data
load:
	@echo "Creating DuckDB database and loading data..."
	@cargo run --bin ff -- demo load

# Run example transformations
transform:
	@echo "Running transformations..."
	@cargo run --bin ff -- demo transform

# Clean up generated files
clean:
	@echo "Cleaning up generated files..."
	@rm -rf demo_project/data/*.csv
	@rm -f demo_project/financial_demo.duckdb

# Generate time-series visualizations
visualize:
	@echo "Generating time-series visualizations..."
	@cargo run --bin ff -- demo visualize