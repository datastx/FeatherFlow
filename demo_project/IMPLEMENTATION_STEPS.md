# FeatherFlow Financial Demo Implementation Steps

This document provides a step-by-step guide for implementing the FeatherFlow Financial Demo. It outlines the necessary changes to the codebase and the creation of required files.

## 1. Add Demo Module to Command Structure

### 1.1. Update `feather_flow/src/commands/mod.rs`

First, add the demo module to the commands module:

```rust
//! CLI commands for FeatherFlow

pub mod parse;
pub mod demo;  // Add this line
```

### 1.2. Create the Demo Module File

Create a new file at `feather_flow/src/commands/demo.rs` which will contain the implementation for all demo-related functionality. The file structure is outlined in `IMPLEMENTATION_PLAN.md`.

## 2. Add the Demo Subcommand to Main CLI

### 2.1. Update `feather_flow/src/main.rs`

Modify the `Command` enum in `main.rs` to include the new `Demo` variant:

```rust
#[derive(Subcommand)]
enum Command {
    /// Parse SQL files and build a dependency graph
    Parse {
        /// Path to the SQL model files
        #[clap(short, long)]
        model_path: PathBuf,

        /// Output format for the graph (dot, text, json)
        #[clap(short, long, default_value = "text")]
        format: String,
    },

    /// Show version information
    Version,
    
    /// Financial demo dataset operations
    Demo {
        #[clap(subcommand)]
        action: DemoAction,
    },
}

#[derive(Subcommand)]
enum DemoAction {
    /// Initialize the demo project structure
    Init,
    
    /// Generate synthetic financial data
    Generate {
        /// Number of customers to generate
        #[clap(short, long, default_value = "100")]
        customers: usize,
        
        /// Number of transactions per account (average)
        #[clap(short, long, default_value = "500")]
        transactions: usize,
        
        /// Time span in days for transaction history
        #[clap(short, long, default_value = "730")]
        days: usize,
    },
    
    /// Load generated data into DuckDB
    Load {
        /// Path to save the DuckDB database
        #[clap(short, long, default_value = "demo_project/financial_demo.duckdb")]
        db_path: PathBuf,
    },
    
    /// Run transformations on the loaded data
    Transform {
        /// Path to the DuckDB database
        #[clap(short, long, default_value = "demo_project/financial_demo.duckdb")]
        db_path: PathBuf,
        
        /// Specific transformation to run (or "all")
        #[clap(short, long, default_value = "all")]
        target: String,
    },
    
    /// Generate visualizations of time-series trends
    Visualize {
        /// Path to the DuckDB database
        #[clap(short, long, default_value = "demo_project/financial_demo.duckdb")]
        db_path: PathBuf,
        
        /// Output directory for visualizations
        #[clap(short, long, default_value = "demo_project/visualizations")]
        output_dir: PathBuf,
    },
}
```

### 2.2. Update the Match Statement in `main.rs`

Add handling for the new `Demo` command:

```rust
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Parse { model_path, format } => {
            // Run the parse command
            if let Err(err) = commands::parse::parse_command(&model_path, &format) {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        }
        Command::Version => {
            // Output version information
            println!("FeatherFlow CLI version {}", env!("CARGO_PKG_VERSION"));
            println!("A Rust-based SQL transformation tool similar to dbt");
            println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        }
        Command::Demo { action } => {
            match action {
                DemoAction::Init => {
                    if let Err(err) = commands::demo::init_command() {
                        eprintln!("Error initializing demo: {}", err);
                        process::exit(1);
                    }
                }
                DemoAction::Generate { customers, transactions, days } => {
                    if let Err(err) = commands::demo::generate_command(customers, transactions, days) {
                        eprintln!("Error generating data: {}", err);
                        process::exit(1);
                    }
                }
                DemoAction::Load { db_path } => {
                    if let Err(err) = commands::demo::load_command(&db_path) {
                        eprintln!("Error loading data: {}", err);
                        process::exit(1);
                    }
                }
                DemoAction::Transform { db_path, target } => {
                    if let Err(err) = commands::demo::transform_command(&db_path, &target) {
                        eprintln!("Error running transformations: {}", err);
                        process::exit(1);
                    }
                }
                DemoAction::Visualize { db_path, output_dir } => {
                    if let Err(err) = commands::demo::visualize_command(&db_path, &output_dir) {
                        eprintln!("Error generating visualizations: {}", err);
                        process::exit(1);
                    }
                }
            }
        }
    }
}
```

## 3. Update Dependencies in Cargo.toml

Add the required dependencies for the demo module to `feather_flow/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
clap = { version = "4.3", features = ["derive"] }
walkdir = "2.3"
sqlparser = "0.30"
# New dependencies for the demo module
chrono = "0.4"
duckdb = "0.8"
rand = "0.8"
rand_distr = "0.4"
csv = "1.2"
serde = { version = "1.0", features = ["derive"] }
```

## 4. Create the Demo Project Structure

### 4.1. Create the Demo Project Directory Structure

```bash
mkdir -p demo_project/data
mkdir -p demo_project/scripts
mkdir -p demo_project/seeds
mkdir -p demo_project/models/staging
mkdir -p demo_project/models/marts/core
mkdir -p demo_project/models/marts/finance
```

### 4.2. Create the Makefile

Create `demo_project/demo_project.mk` with the contents from the `IMPLEMENTATION_PLAN.md` document.

### 4.3. Create Seed Data Files

Create the following seed files to help with data generation:

**`demo_project/seeds/merchant_categories.csv`**:
```csv
category,online_probability,popularity_score
Grocery,0.2,0.95
Dining,0.3,0.9
Coffee Shops,0.1,0.85
Entertainment,0.5,0.8
Travel,0.7,0.6
Clothing,0.6,0.75
Electronics,0.8,0.7
Home Improvement,0.4,0.65
Healthcare,0.3,0.5
Insurance,0.7,0.4
Utilities,0.8,0.9
Telecommunications,0.9,0.85
Transportation,0.5,0.7
Education,0.6,0.5
Financial Services,0.7,0.6
```

**`demo_project/seeds/transaction_types.csv`**:
```csv
type,description,is_recurring_probability,category
deposit,Direct Deposit,0.9,Income
withdrawal,ATM Withdrawal,0.1,Cash
payment,Debit Card Purchase,0.0,Purchase
transfer,Account Transfer,0.2,Transfer
fee,Account Fee,0.8,Fee
bill_payment,Utility Bill Payment,0.9,Bill
subscription,Monthly Subscription,0.95,Subscription
refund,Purchase Refund,0.0,Refund
interest,Account Interest,0.9,Interest
loan_payment,Loan Payment,0.9,Loan
```

## 5. Create SQL Model Files

Create the SQL model files as detailed in the `SQL_TRANSFORMATIONS.md` file. These files should be placed in the appropriate directories under `demo_project/models/`.

## 6. Implement the Demo Module

### 6.1. Core Implementation Structure

The `demo.rs` module should implement the following functions:

1. `init_command()`: Create the directory structure and example files
2. `generate_command()`: Generate synthetic financial data
3. `load_command()`: Load data into DuckDB
4. `transform_command()`: Run transformations
5. `visualize_command()`: Generate visualizations

### 6.2. Helper Functions

Additionally, implement these helper functions:

1. Data generation functions:
   - `generate_customers()`
   - `generate_merchants()`
   - `generate_accounts()`
   - `generate_credit_cards()`
   - `generate_loans()`
   - `generate_transactions()`

2. Data loading functions:
   - `write_to_csv()`
   - `create_and_load_table()`
   - `run_sql_model()`

## 7. Testing the Implementation

### 7.1. Build the Project

```bash
cd feather_flow
cargo build
```

### 7.2. Initialize the Demo

```bash
cargo run -- demo init
```

### 7.3. Generate Test Data

```bash
cargo run -- demo generate --customers 20 --transactions 100 --days 365
```

### 7.4. Load Data into DuckDB

```bash
cargo run -- demo load
```

### 7.5. Run Transformations

```bash
cargo run -- demo transform
```

## 8. Future Extensions

Once the basic implementation is working, consider these extensions:

1. Add interactive visualizations using a Rust plotting library
2. Implement anomaly detection in transaction patterns
3. Add dashboard functionality for exploring the data
4. Create additional financial metrics and KPIs

## Implementation Timeline

1. **Day 1**: Set up project structure, create demo.rs module and update CLI
2. **Day 2**: Implement data generation functions
3. **Day 3**: Implement data loading and transformations
4. **Day 4**: Write tests and documentation
5. **Day 5**: Implement visualizations and polish the implementation

This implementation plan provides a step-by-step guide to create a comprehensive financial demo for FeatherFlow with rich time-series features.