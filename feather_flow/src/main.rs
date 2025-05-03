use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod commands;
mod sql_engine;

/// FeatherFlow (ff) CLI - SQL transformation tool
#[derive(Parser)]
#[clap(name = "ff", about = "FeatherFlow - SQL transformation tool", version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

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
        Command::Demo { action } => match action {
            DemoAction::Init => {
                if let Err(err) = commands::demo::init_command() {
                    eprintln!("Error initializing demo: {}", err);
                    process::exit(1);
                }
            }
            DemoAction::Generate {
                customers,
                transactions,
                days,
            } => {
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
            DemoAction::Visualize {
                db_path,
                output_dir,
            } => {
                if let Err(err) = commands::demo::visualize_command(&db_path, &output_dir) {
                    eprintln!("Error generating visualizations: {}", err);
                    process::exit(1);
                }
            }
        },
    }
}
