//! Financial demo dataset module for FeatherFlow

use std::error::Error;
use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::path::Path;

/// Initialize the demo project structure
pub fn init_command() -> Result<(), Box<dyn Error>> {
    println!("Initializing demo project structure...");

    // Create directories
    create_directory_structure()?;

    // Create example SQL models
    create_example_sql_models()?;

    println!("Demo project initialized successfully!");
    Ok(())
}

/// Generate synthetic financial data
pub fn generate_command(
    customers: usize,
    transactions_per_account: usize,
    days: usize,
) -> Result<(), Box<dyn Error>> {
    println!("Generating synthetic financial data...");
    println!(
        "Parameters: {} customers, ~{} transactions per account, {} days of history",
        customers, transactions_per_account, days
    );

    // This is a placeholder - we'll implement the actual data generation logic later
    println!("Data generation would create synthetic financial data with time-series patterns.");
    println!("For now, this is a placeholder until we implement the full generator.");

    // Create data directory
    create_dir_all("demo_project/data")?;

    // Create example CSV files with minimal data
    create_example_csv_files(customers)?;

    println!("Created example CSV files in demo_project/data/");
    Ok(())
}

/// Load generated data into DuckDB
pub fn load_command(db_path: &Path) -> Result<(), Box<dyn Error>> {
    println!("Loading data into a database at: {}", db_path.display());

    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        create_dir_all(parent)?;
    }

    // This is a simplified version - we're just creating a placeholder database file
    println!("Database loading would create a database and import CSV files.");
    println!("For now, this is a placeholder that creates an empty file to simulate a database.");

    // Create an empty file to represent the database
    File::create(db_path)?;

    println!("Created empty database file at {}", db_path.display());
    Ok(())
}

/// Run transformations on the loaded data
pub fn transform_command(db_path: &Path, target: &str) -> Result<(), Box<dyn Error>> {
    println!("Running transformations on data in: {}", db_path.display());
    println!("Target transformation: {}", target);

    // This is a simplified version for demo purposes
    println!("In a full implementation, this would execute SQL models against a database.");
    println!("The SQL files have been created in the models/ directory and can be viewed.");

    let target_dir = match target {
        "staging" => "demo_project/models/staging",
        "core" => "demo_project/models/marts/core",
        "finance" => "demo_project/models/marts/finance",
        _ => "demo_project/models",
    };

    // List the SQL files in the target directory
    let paths = fs::read_dir(target_dir)?;
    println!("\nSQL files in {}:", target_dir);
    for path in paths {
        let path = path?;
        println!("  - {}", path.file_name().to_string_lossy());
    }

    println!("Transformation '{}' completed successfully!", target);
    Ok(())
}

/// Generate visualizations of time-series trends
pub fn visualize_command(db_path: &Path, output_dir: &Path) -> Result<(), Box<dyn Error>> {
    println!(
        "Generating visualizations from data in: {}",
        db_path.display()
    );
    println!("Output directory: {}", output_dir.display());

    // Create output directory
    create_dir_all(output_dir)?;

    // This is a placeholder - we'll implement the actual visualization logic later
    println!("Visualization would generate charts and graphs from the transformed data.");
    println!("For now, this is a placeholder until we implement the full visualizer.");

    // Create an example text file
    let visualization_file = output_dir.join("example_visualization.txt");
    let mut file = File::create(visualization_file)?;
    writeln!(
        file,
        "This is a placeholder for actual visualization output."
    )?;

    println!(
        "Created example visualization output in {}",
        output_dir.display()
    );
    Ok(())
}

// Helper function to create the directory structure
fn create_directory_structure() -> Result<(), Box<dyn Error>> {
    // Create the main directories
    create_dir_all("demo_project/data")?;
    create_dir_all("demo_project/models/staging")?;
    create_dir_all("demo_project/models/marts/core")?;
    create_dir_all("demo_project/models/marts/finance")?;
    create_dir_all("demo_project/seeds")?;

    // Create the seeds directory with example seed files
    create_merchant_categories_seed()?;
    create_transaction_types_seed()?;

    Ok(())
}

// Helper function to create example SQL models
fn create_example_sql_models() -> Result<(), Box<dyn Error>> {
    // Staging models
    create_example_sql_file("demo_project/models/staging/stg_customers.sql", 
        "SELECT\n    customer_id,\n    name,\n    email,\n    address,\n    registration_date,\n    credit_score,\n    income_bracket\nFROM raw_data.customers")?;

    create_example_sql_file("demo_project/models/staging/stg_accounts.sql", 
        "SELECT\n    account_id,\n    customer_id,\n    account_type,\n    open_date,\n    status,\n    currency,\n    initial_balance,\n    current_balance\nFROM raw_data.accounts")?;

    create_example_sql_file("demo_project/models/staging/stg_transactions.sql", 
        "SELECT\n    transaction_id,\n    account_id,\n    merchant_id,\n    transaction_datetime,\n    amount,\n    transaction_type,\n    description,\n    category,\n    status,\n    is_recurring,\n    day_of_week,\n    month,\n    year,\n    time_of_day\nFROM raw_data.transactions")?;

    create_example_sql_file("demo_project/models/staging/stg_merchants.sql", 
        "SELECT\n    merchant_id,\n    name,\n    category,\n    location,\n    is_online,\n    popularity_score\nFROM raw_data.merchants")?;

    // Mart models - core
    create_example_sql_file("demo_project/models/marts/core/customer_summary.sql", 
        "SELECT\n    c.customer_id,\n    c.name,\n    c.email,\n    c.credit_score,\n    COUNT(DISTINCT a.account_id) AS account_count,\n    SUM(a.current_balance) AS total_balance,\n    COUNT(DISTINCT t.transaction_id) AS transaction_count,\n    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,\n    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income\nFROM staging.stg_customers c\nLEFT JOIN staging.stg_accounts a ON c.customer_id = a.customer_id\nLEFT JOIN staging.stg_transactions t ON a.account_id = t.account_id\nGROUP BY c.customer_id, c.name, c.email, c.credit_score")?;

    // Mart models - finance
    create_example_sql_file("demo_project/models/marts/finance/daily_trends.sql", 
        "SELECT\n    DATE(t.transaction_datetime) AS date,\n    t.day_of_week,\n    COUNT(*) AS transaction_count,\n    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,\n    COUNT(DISTINCT t.account_id) AS active_accounts\nFROM staging.stg_transactions t\nGROUP BY DATE(t.transaction_datetime), t.day_of_week\nORDER BY date")?;

    create_example_sql_file("demo_project/models/marts/finance/monthly_trends.sql", 
        "SELECT\n    t.year,\n    t.month,\n    COUNT(*) AS transaction_count,\n    SUM(CASE WHEN t.amount < 0 THEN ABS(t.amount) ELSE 0 END) AS total_spending,\n    SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END) AS total_income\nFROM staging.stg_transactions t\nGROUP BY t.year, t.month\nORDER BY t.year, t.month")?;

    Ok(())
}

// Helper function to create an example SQL file
fn create_example_sql_file(path: &str, content: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Helper function to create merchant categories seed file
fn create_merchant_categories_seed() -> Result<(), Box<dyn Error>> {
    let content = "category,online_probability,popularity_score\nGrocery,0.2,0.95\nDining,0.3,0.9\nCoffee Shops,0.1,0.85\nEntertainment,0.5,0.8\nTravel,0.7,0.6\nClothing,0.6,0.75\nElectronics,0.8,0.7\nHome Improvement,0.4,0.65\nHealthcare,0.3,0.5\nInsurance,0.7,0.4\nUtilities,0.8,0.9\nTelecommunications,0.9,0.85\nTransportation,0.5,0.7\nEducation,0.6,0.5\nFinancial Services,0.7,0.6";

    create_dir_all("demo_project/seeds")?;
    let mut file = File::create("demo_project/seeds/merchant_categories.csv")?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

// Helper function to create transaction types seed file
fn create_transaction_types_seed() -> Result<(), Box<dyn Error>> {
    let content = "type,description,is_recurring_probability,category\ndeposit,Direct Deposit,0.9,Income\nwithdrawal,ATM Withdrawal,0.1,Cash\npayment,Debit Card Purchase,0.0,Purchase\ntransfer,Account Transfer,0.2,Transfer\nfee,Account Fee,0.8,Fee\nbill_payment,Utility Bill Payment,0.9,Bill\nsubscription,Monthly Subscription,0.95,Subscription\nrefund,Purchase Refund,0.0,Refund\ninterest,Account Interest,0.9,Interest\nloan_payment,Loan Payment,0.9,Loan";

    let mut file = File::create("demo_project/seeds/transaction_types.csv")?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

// Helper function to create example CSV files
fn create_example_csv_files(customer_count: usize) -> Result<(), Box<dyn Error>> {
    // Create customers.csv
    let mut customers_content = String::from(
        "customer_id,name,email,address,registration_date,credit_score,income_bracket\n",
    );
    for i in 1..=customer_count {
        customers_content.push_str(&format!(
            "{},Customer {},customer{}@example.com,\"123 Main St, City\",2023-01-{:02},{},({})\n",
            i,
            i,
            i,
            i % 28 + 1,
            650 + i % 200,
            if i % 3 == 0 {
                "High"
            } else if i % 3 == 1 {
                "Medium"
            } else {
                "Low"
            }
        ));
    }

    let mut file = File::create("demo_project/data/customers.csv")?;
    file.write_all(customers_content.as_bytes())?;

    // Create accounts.csv (1-2 accounts per customer)
    let mut accounts_content = String::from("account_id,customer_id,account_type,open_date,status,currency,initial_balance,current_balance\n");
    let mut account_id = 1;
    for i in 1..=customer_count {
        // Each customer has 1-2 accounts
        let account_count = 1 + i % 2;
        for j in 0..account_count {
            accounts_content.push_str(&format!(
                "{},{},({}),2023-02-{:02},Active,USD,{:.2},{:.2}\n",
                account_id,
                i,
                if j == 0 { "Checking" } else { "Savings" },
                i % 28 + 1,
                1000.0 + (i * 100) as f64,
                1200.0 + (i * 100) as f64
            ));
            account_id += 1;
        }
    }

    let mut file = File::create("demo_project/data/accounts.csv")?;
    file.write_all(accounts_content.as_bytes())?;

    // Create merchants.csv
    let mut merchants_content =
        String::from("merchant_id,name,category,location,is_online,popularity_score\n");
    let merchant_categories = [
        "Grocery",
        "Dining",
        "Coffee",
        "Entertainment",
        "Travel",
        "Clothing",
        "Electronics",
    ];
    for i in 1..=50 {
        let category = merchant_categories[i % merchant_categories.len()];
        merchants_content.push_str(&format!(
            "{},(Merchant {}),{},\"456 Commerce St, City\",{},{:.1}\n",
            i,
            i,
            category,
            if i % 2 == 0 { "true" } else { "false" },
            0.5 + (i % 10) as f64 / 10.0
        ));
    }

    let mut file = File::create("demo_project/data/merchants.csv")?;
    file.write_all(merchants_content.as_bytes())?;

    // Create minimal transactions.csv
    let mut transactions_content = String::from("transaction_id,account_id,merchant_id,transaction_datetime,amount,transaction_type,description,category,status,is_recurring,day_of_week,month,year,time_of_day\n");
    let mut transaction_id = 1;

    for account_id in 1..account_id {
        // Generate 10 transactions per account as a minimal example
        for i in 0..10 {
            // Generate transaction data
            let is_income = i % 3 == 0;
            let amount = if is_income {
                500.0 + (i as f64 * 10.0)
            } else {
                -(50.0 + (i as f64 * 5.0))
            };

            // Generate date components
            let month = 1 + (i % 12);
            let day = 1 + (i % 28);
            let year = 2023;
            let hour = 8 + (i % 12);

            // Generate other transaction attributes
            let merchant_id = 1 + (i % 50);
            let day_of_week = i % 7;
            let transaction_type = if is_income { "deposit" } else { "payment" };
            let description = if is_income {
                "Income Payment"
            } else {
                "Purchase"
            };
            let category = if is_income {
                "Income"
            } else {
                merchant_categories[i % merchant_categories.len()]
            };
            let is_recurring = if i % 5 == 0 { "true" } else { "false" };
            let time_of_day = if hour < 12 {
                "morning"
            } else if hour < 17 {
                "afternoon"
            } else {
                "evening"
            };

            // Format the transaction row
            let row =
                format!(
                "{},{},{},{:04}-{:02}-{:02} {:02}:00:00,{:.2},{},{},{},Completed,{},{},{},{},{}\n",
                transaction_id, account_id, merchant_id,
                year, month, day, hour,
                amount, transaction_type, description, category,
                is_recurring, day_of_week, month, year, time_of_day
            );

            transactions_content.push_str(&row);
            transaction_id += 1;
        }
    }

    let mut file = File::create("demo_project/data/transactions.csv")?;
    file.write_all(transactions_content.as_bytes())?;

    Ok(())
}
