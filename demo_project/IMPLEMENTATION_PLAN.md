# FeatherFlow Financial Demo Implementation Plan

This document outlines the complete implementation plan for the FeatherFlow Financial Demo project, which will create a synthetic financial dataset with time-series features and demonstrate FeatherFlow's capabilities with financial data transformations.

## 1. Project Structure

```
demo_project/
├── demo_project.mk                   # Main Makefile for the demo
├── data/                             # Generated data directory
├── scripts/                          # Scripts directory
│   └── generate_financial_data.rs    # Data generation script
├── seeds/                            # Seed data for consistent generation
│   ├── merchant_categories.csv       # Merchant categories
│   └── transaction_types.csv         # Transaction types and descriptions
└── models/                           # SQL transformation models
    ├── staging/                      # Staging models
    │   ├── stg_customers.sql
    │   ├── stg_accounts.sql
    │   ├── stg_transactions.sql
    │   ├── stg_merchants.sql
    │   ├── stg_credit_cards.sql
    │   └── stg_loans.sql
    └── marts/                        # Business-level transformations
        ├── core/
        │   ├── customer_summary.sql  # Customer account aggregates
        │   └── merchant_summary.sql  # Merchant transaction aggregates
        └── finance/
            ├── daily_trends.sql      # Daily transaction patterns
            ├── monthly_trends.sql    # Monthly aggregations and trends
            ├── spending_categories.sql # Spending by category
            └── recurring_analysis.sql # Analysis of recurring transactions
```

## 2. File Implementations

### 2.1. Makefile (`demo_project.mk`)

```makefile
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
```

### 2.2. Rust CLI Extension (`feather_flow/src/commands/demo.rs`)

```rust
use std::error::Error;
use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};
use duckdb::{Connection, params};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::distributions::{Distribution, Uniform};
use rand_distr::{Normal, LogNormal};
use csv::Writer;
use serde::Serialize;

// Data structures for our financial entities
#[derive(Serialize)]
struct Customer {
    customer_id: i32,
    name: String,
    email: String,
    address: String,
    registration_date: String, // ISO format
    credit_score: i32,
    income_bracket: String,
}

#[derive(Serialize)]
struct Account {
    account_id: i32,
    customer_id: i32,
    account_type: String,
    open_date: String, // ISO format
    status: String,
    currency: String,
    initial_balance: f64,
    current_balance: f64,
}

#[derive(Serialize)]
struct Transaction {
    transaction_id: i32,
    account_id: i32,
    merchant_id: i32,
    card_id: Option<i32>,
    transaction_datetime: String, // ISO format
    amount: f64,
    transaction_type: String,
    description: String,
    category: String,
    status: String,
    is_recurring: bool,
    day_of_week: i32,
    month: i32,
    year: i32,
    time_of_day: String,
}

#[derive(Serialize)]
struct Merchant {
    merchant_id: i32,
    name: String,
    category: String,
    location: String,
    is_online: bool,
    popularity_score: f32,
}

#[derive(Serialize)]
struct CreditCard {
    card_id: i32,
    customer_id: i32,
    card_type: String,
    credit_limit: f64,
    issue_date: String, // ISO format
    expiry_date: String, // ISO format
    status: String,
}

#[derive(Serialize)]
struct Loan {
    loan_id: i32,
    customer_id: i32,
    loan_type: String,
    principal_amount: f64,
    interest_rate: f64,
    term_months: i32,
    start_date: String, // ISO format
    status: String,
}

/// Initialize the demo project structure
pub fn init_command() -> Result<(), Box<dyn Error>> {
    println!("Initializing demo project structure...");
    
    // Create directories
    create_directories()?;
    
    // Create seed data
    create_seed_data()?;
    
    // Create SQL model files
    create_sql_models()?;
    
    println!("Demo project initialized successfully!");
    Ok(())
}

/// Generate synthetic financial data
pub fn generate_command(customers: usize, transactions_per_account: usize, days: usize) -> Result<(), Box<dyn Error>> {
    println!("Generating synthetic financial data...");
    println!("Parameters: {} customers, ~{} transactions per account, {} days of history", 
             customers, transactions_per_account, days);
    
    // Create data directory if it doesn't exist
    create_dir_all("demo_project/data")?;
    
    // Use a fixed seed for deterministic generation
    let seed = 42;
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Generate customers
    let customers_data = generate_customers(&mut rng, customers)?;
    
    // Generate merchants
    let merchants_data = generate_merchants(&mut rng, 500)?;
    
    // Generate accounts (approximately 1-3 per customer)
    let accounts_data = generate_accounts(&mut rng, &customers_data)?;
    
    // Generate credit cards
    let credit_cards_data = generate_credit_cards(&mut rng, &customers_data)?;
    
    // Generate loans
    let loans_data = generate_loans(&mut rng, &customers_data)?;
    
    // Generate transactions with time-series data
    let end_date = Utc::now().naive_utc().date();
    let start_date = end_date - Duration::days(days as i64);
    
    let transactions_data = generate_transactions(
        &mut rng, 
        &accounts_data, 
        &merchants_data, 
        &credit_cards_data,
        start_date,
        end_date,
        transactions_per_account
    )?;
    
    // Write generated data to CSV files
    write_to_csv("demo_project/data/customers.csv", &customers_data)?;
    write_to_csv("demo_project/data/merchants.csv", &merchants_data)?;
    write_to_csv("demo_project/data/accounts.csv", &accounts_data)?;
    write_to_csv("demo_project/data/credit_cards.csv", &credit_cards_data)?;
    write_to_csv("demo_project/data/loans.csv", &loans_data)?;
    write_to_csv("demo_project/data/transactions.csv", &transactions_data)?;
    
    println!("Data generation completed successfully!");
    println!("Generated:");
    println!("  - {} customers", customers_data.len());
    println!("  - {} merchants", merchants_data.len());
    println!("  - {} accounts", accounts_data.len());
    println!("  - {} credit cards", credit_cards_data.len());
    println!("  - {} loans", loans_data.len());
    println!("  - {} transactions", transactions_data.len());
    
    Ok(())
}

/// Load generated data into DuckDB
pub fn load_command(db_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("Loading data into DuckDB at: {}", db_path.display());
    
    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        create_dir_all(parent)?;
    }
    
    // Connect to DuckDB
    let conn = Connection::open(db_path)?;
    
    // Create schema
    conn.execute("CREATE SCHEMA IF NOT EXISTS raw_data", params![])?;
    conn.execute("CREATE SCHEMA IF NOT EXISTS staging", params![])?;
    conn.execute("CREATE SCHEMA IF NOT EXISTS marts", params![])?;
    
    // Create tables and load data
    create_and_load_table(&conn, "customers")?;
    create_and_load_table(&conn, "merchants")?;
    create_and_load_table(&conn, "accounts")?;
    create_and_load_table(&conn, "credit_cards")?;
    create_and_load_table(&conn, "loans")?;
    create_and_load_table(&conn, "transactions")?;
    
    println!("Data loaded successfully into DuckDB!");
    Ok(())
}

/// Run transformations on the loaded data
pub fn transform_command(db_path: &PathBuf, target: &str) -> Result<(), Box<dyn Error>> {
    println!("Running transformations on data in: {}", db_path.display());
    
    // Connect to DuckDB
    let conn = Connection::open(db_path)?;
    
    // Run staging models
    if target == "all" || target == "staging" {
        run_sql_model(&conn, "models/staging/stg_customers.sql")?;
        run_sql_model(&conn, "models/staging/stg_accounts.sql")?;
        run_sql_model(&conn, "models/staging/stg_transactions.sql")?;
        run_sql_model(&conn, "models/staging/stg_merchants.sql")?;
        run_sql_model(&conn, "models/staging/stg_credit_cards.sql")?;
        run_sql_model(&conn, "models/staging/stg_loans.sql")?;
        println!("Staging models completed.");
    }
    
    // Run core mart models
    if target == "all" || target == "core" {
        run_sql_model(&conn, "models/marts/core/customer_summary.sql")?;
        run_sql_model(&conn, "models/marts/core/merchant_summary.sql")?;
        println!("Core mart models completed.");
    }
    
    // Run finance mart models
    if target == "all" || target == "finance" {
        run_sql_model(&conn, "models/marts/finance/daily_trends.sql")?;
        run_sql_model(&conn, "models/marts/finance/monthly_trends.sql")?;
        run_sql_model(&conn, "models/marts/finance/spending_categories.sql")?;
        run_sql_model(&conn, "models/marts/finance/recurring_analysis.sql")?;
        println!("Finance mart models completed.");
    }
    
    println!("Transformations completed successfully!");
    Ok(())
}

/// Generate visualizations of time-series trends
pub fn visualize_command(db_path: &PathBuf, output_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("Generating visualizations from data in: {}", db_path.display());
    
    // Create output directory
    create_dir_all(output_dir)?;
    
    // This function would use a visualization library (like plotters)
    // to generate time-series charts from the transformed data
    
    println!("This feature would generate time-series visualizations from the transformed data.");
    println!("For now, you can query the DuckDB database directly to view the transformed data.");
    
    Ok(())
}

// Helper functions for data generation and loading would be implemented here
// These would include the actual implementations of:
// - create_directories
// - create_seed_data
// - create_sql_models
// - generate_customers
// - generate_merchants
// - generate_accounts
// - generate_credit_cards
// - generate_loans
// - generate_transactions
// - write_to_csv
// - create_and_load_table
// - run_sql_model
```

### 2.3. CLI Integration (`feather_flow/src/main.rs` changes)

Add the following to `main.rs`:

```rust
mod commands;

#[derive(Subcommand)]
enum Command {
    // Existing commands...
    
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

// In the main function, add handling for the Demo command:
fn main() {
    // ... existing code ...
    
    match cli.command {
        // ... existing commands ...
        
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

### 2.4. SQL Model Files

#### Staging Models

**`models/staging/stg_customers.sql`**:
```sql
SELECT
    customer_id,
    name,
    email,
    address,
    registration_date,
    credit_score,
    income_bracket
FROM raw_data.customers
```

**`models/staging/stg_accounts.sql`**:
```sql
SELECT
    account_id,
    customer_id,
    account_type,
    open_date,
    status,
    currency,
    initial_balance,
    current_balance
FROM raw_data.accounts
```

**`models/staging/stg_transactions.sql`**:
```sql
SELECT
    transaction_id,
    account_id,
    merchant_id,
    card_id,
    transaction_datetime,
    amount,
    transaction_type,
    description,
    category,
    status,
    is_recurring,
    -- Extract time components for analysis
    day_of_week,
    month,
    year,
    time_of_day
FROM raw_data.transactions
```

**`models/staging/stg_merchants.sql`**:
```sql
SELECT
    merchant_id,
    name,
    category,
    location,
    is_online,
    popularity_score
FROM raw_data.merchants
```

**`models/staging/stg_credit_cards.sql`**:
```sql
SELECT
    card_id,
    customer_id,
    card_type,
    credit_limit,
    issue_date,
    expiry_date,
    status
FROM raw_data.credit_cards
```

**`models/staging/stg_loans.sql`**:
```sql
SELECT
    loan_id,
    customer_id,
    loan_type,
    principal_amount,
    interest_rate,
    term_months,
    start_date,
    status
FROM raw_data.loans
```

#### Mart Models - Core

**`models/marts/core/customer_summary.sql`**:
```sql
SELECT
    c.customer_id,
    c.name,
    c.email,
    c.credit_score,
    c.income_bracket,
    -- Account summary
    COUNT(DISTINCT a.account_id) AS account_count,
    SUM(a.current_balance) AS total_balance,
    -- Transaction summary
    COUNT(DISTINCT t.transaction_id) AS transaction_count,
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income,
    COUNT(DISTINCT t.category) AS spending_category_count,
    -- Credit info
    COUNT(DISTINCT cc.card_id) AS credit_card_count,
    SUM(cc.credit_limit) AS total_credit_limit,
    -- Loan info
    COUNT(DISTINCT l.loan_id) AS loan_count,
    SUM(l.principal_amount) AS total_loan_amount
FROM staging.stg_customers c
LEFT JOIN staging.stg_accounts a ON c.customer_id = a.customer_id
LEFT JOIN staging.stg_transactions t ON a.account_id = t.account_id
LEFT JOIN staging.stg_credit_cards cc ON c.customer_id = cc.customer_id
LEFT JOIN staging.stg_loans l ON c.customer_id = l.customer_id
GROUP BY c.customer_id, c.name, c.email, c.credit_score, c.income_bracket
```

**`models/marts/core/merchant_summary.sql`**:
```sql
SELECT
    m.merchant_id,
    m.name,
    m.category,
    m.location,
    m.is_online,
    -- Transaction metrics
    COUNT(t.transaction_id) AS transaction_count,
    COUNT(DISTINCT t.account_id) AS unique_customer_count,
    SUM(ABS(t.amount)) AS transaction_volume,
    AVG(ABS(t.amount)) AS avg_transaction_amount,
    -- Time patterns
    AVG(CASE WHEN t.day_of_week IN (0, 6) THEN ABS(t.amount) ELSE NULL END) AS avg_weekend_amount,
    AVG(CASE WHEN t.day_of_week BETWEEN 1 AND 5 THEN ABS(t.amount) ELSE NULL END) AS avg_weekday_amount,
    -- Recurring metrics
    SUM(CASE WHEN t.is_recurring THEN 1 ELSE 0 END) AS recurring_transaction_count,
    -- Monthly stats
    COUNT(DISTINCT (t.year || '-' || t.month)) AS active_months
FROM staging.stg_merchants m
LEFT JOIN staging.stg_transactions t ON m.merchant_id = t.merchant_id
GROUP BY m.merchant_id, m.name, m.category, m.location, m.is_online
```

#### Mart Models - Finance

**`models/marts/finance/daily_trends.sql`**:
```sql
SELECT
    DATE(t.transaction_datetime) AS date,
    t.day_of_week,
    COUNT(t.transaction_id) AS transaction_count,
    COUNT(DISTINCT t.account_id) AS active_accounts,
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income,
    AVG(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE NULL END) AS avg_spend_amount,
    -- Day over day metrics
    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) - 
        LAG(SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END)) OVER 
        (ORDER BY DATE(t.transaction_datetime)) AS spending_change_from_previous_day,
    -- Number of merchant categories
    COUNT(DISTINCT m.category) AS active_merchant_categories
FROM staging.stg_transactions t
JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
GROUP BY DATE(t.transaction_datetime), t.day_of_week
ORDER BY date
```

**`models/marts/finance/monthly_trends.sql`**:
```sql
WITH monthly_data AS (
    SELECT
        t.year,
        t.month,
        DATE_TRUNC('month', t.transaction_datetime) AS month_start,
        COUNT(t.transaction_id) AS transaction_count,
        COUNT(DISTINCT t.account_id) AS active_accounts,
        SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,
        SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income,
        AVG(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE NULL END) AS avg_spend_amount,
        COUNT(DISTINCT m.category) AS category_count
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    GROUP BY t.year, t.month, DATE_TRUNC('month', t.transaction_datetime)
)

SELECT
    d.year,
    d.month,
    d.month_start,
    d.transaction_count,
    d.active_accounts,
    d.total_spending,
    d.total_income,
    d.avg_spend_amount,
    d.category_count,
    -- Month over month growth
    d.total_spending - LAG(d.total_spending) OVER (ORDER BY d.year, d.month) AS mom_spending_change,
    (d.total_spending - LAG(d.total_spending) OVER (ORDER BY d.year, d.month)) / 
        NULLIF(LAG(d.total_spending) OVER (ORDER BY d.year, d.month), 0) * 100 AS mom_spending_pct_change,
    -- Year over year comparison (for same month)
    d.total_spending - LAG(d.total_spending) OVER (
        PARTITION BY d.month ORDER BY d.year
    ) AS yoy_spending_change,
    (d.total_spending - LAG(d.total_spending) OVER (
        PARTITION BY d.month ORDER BY d.year
    )) / NULLIF(LAG(d.total_spending) OVER (
        PARTITION BY d.month ORDER BY d.year
    ), 0) * 100 AS yoy_spending_pct_change
FROM monthly_data d
ORDER BY d.year, d.month
```

**`models/marts/finance/spending_categories.sql`**:
```sql
WITH category_spending AS (
    SELECT
        t.year,
        t.month,
        DATE_TRUNC('month', t.transaction_datetime) AS month_start,
        m.category,
        COUNT(t.transaction_id) AS transaction_count,
        SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS category_spending
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    WHERE t.amount < 0  -- Only outgoing transactions
    GROUP BY t.year, t.month, DATE_TRUNC('month', t.transaction_datetime), m.category
),

monthly_totals AS (
    SELECT
        year,
        month,
        month_start,
        SUM(category_spending) AS total_monthly_spending
    FROM category_spending
    GROUP BY year, month, month_start
)

SELECT
    cs.year,
    cs.month,
    cs.month_start,
    cs.category,
    cs.transaction_count,
    cs.category_spending,
    -- Category percentage of monthly spending
    cs.category_spending / NULLIF(mt.total_monthly_spending, 0) * 100 AS category_pct_of_total,
    -- Rank categories within each month
    ROW_NUMBER() OVER (PARTITION BY cs.year, cs.month ORDER BY cs.category_spending DESC) AS category_rank,
    -- Previous year's spending in same category and month
    LAG(cs.category_spending) OVER (
        PARTITION BY cs.month, cs.category ORDER BY cs.year
    ) AS prev_year_category_spending,
    -- Year over year category growth
    (cs.category_spending - LAG(cs.category_spending) OVER (
        PARTITION BY cs.month, cs.category ORDER BY cs.year
    )) / NULLIF(LAG(cs.category_spending) OVER (
        PARTITION BY cs.month, cs.category ORDER BY cs.year
    ), 0) * 100 AS yoy_category_pct_change
FROM category_spending cs
JOIN monthly_totals mt ON cs.year = mt.year AND cs.month = mt.month
ORDER BY cs.year, cs.month, category_spending DESC
```

**`models/marts/finance/recurring_analysis.sql`**:
```sql
WITH recurring_transactions AS (
    SELECT
        t.account_id,
        t.merchant_id,
        m.name AS merchant_name,
        m.category AS merchant_category,
        COUNT(t.transaction_id) AS transaction_count,
        MIN(ABS(t.amount)) AS min_amount,
        MAX(ABS(t.amount)) AS max_amount,
        AVG(ABS(t.amount)) AS avg_amount,
        MIN(t.transaction_datetime) AS first_transaction,
        MAX(t.transaction_datetime) AS last_transaction,
        -- Calculate time between transactions
        (DATEDIFF('day', MIN(t.transaction_datetime), MAX(t.transaction_datetime))) / 
            NULLIF(COUNT(t.transaction_id) - 1, 0) AS avg_days_between
    FROM staging.stg_transactions t
    JOIN staging.stg_merchants m ON t.merchant_id = m.merchant_id
    WHERE t.is_recurring = TRUE
    GROUP BY t.account_id, t.merchant_id, m.name, m.category
    HAVING COUNT(t.transaction_id) > 1  -- Must have more than one transaction
),

account_summary AS (
    SELECT
        rt.account_id,
        COUNT(DISTINCT rt.merchant_id) AS recurring_merchant_count,
        SUM(rt.avg_amount) AS total_recurring_monthly_expenses
    FROM recurring_transactions rt
    WHERE rt.avg_days_between BETWEEN 28 AND 31  -- Monthly recurring
    GROUP BY rt.account_id
)

SELECT
    c.customer_id,
    c.name AS customer_name,
    a.account_id,
    a.account_type,
    asm.recurring_merchant_count,
    asm.total_recurring_monthly_expenses,
    -- Income metrics for comparison
    (SELECT SUM(t.amount) / COUNT(DISTINCT (t.year || '-' || t.month))
     FROM staging.stg_transactions t
     WHERE t.account_id = a.account_id AND t.amount > 0) AS avg_monthly_income,
    -- Ratio of recurring expenses to income
    asm.total_recurring_monthly_expenses / NULLIF(
        (SELECT SUM(t.amount) / COUNT(DISTINCT (t.year || '-' || t.month))
         FROM staging.stg_transactions t
         WHERE t.account_id = a.account_id AND t.amount > 0), 0
    ) * 100 AS recurring_expense_to_income_pct
FROM staging.stg_customers c
JOIN staging.stg_accounts a ON c.customer_id = a.customer_id
JOIN account_summary asm ON a.account_id = asm.account_id
ORDER BY recurring_expense_to_income_pct DESC
```

## 3. Rust Dependencies Required

The following dependencies need to be added to the `Cargo.toml` file:

```toml
[dependencies]
# Other existing dependencies...
chrono = "0.4"
duckdb = "0.8"
rand = "0.8"
rand_distr = "0.4"
csv = "1.2"
serde = { version = "1.0", features = ["derive"] }
```

## 4. Implementation Strategy

1. First, create the root directory structure and files using the switch_mode tool to change to Code mode
2. Add the demo.rs module to the commands directory  
3. Update main.rs to include the new subcommand
4. Create the SQL model files
5. Test the implementation

## 5. Future Extensions

After the basic implementation, we could consider these extensions:

1. Add interactive visualizations for the time-series data
2. Implement anomaly detection in transaction patterns
3. Add benchmark tools to measure transformation performance
4. Create a UI dashboard for exploring the financial data

This implementation plan provides a comprehensive blueprint for building a financial demo dataset with time-series features for FeatherFlow.