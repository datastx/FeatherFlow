use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod commands;
mod sql_engine;
mod validators;

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

        /// Skip validation of model file structure
        #[clap(short, long)]
        skip_validation: bool,
    },

    /// Validate model file structure
    Validate {
        /// Path to the SQL model files
        #[clap(short, long)]
        model_path: PathBuf,

        /// Quiet mode - only output errors
        #[clap(short, long)]
        quiet: bool,
    },

    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Parse {
            model_path,
            format,
            skip_validation,
        } => {
            // Run the parse command
            if let Err(err) = commands::parse::parse_command(&model_path, &format, !skip_validation) {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        }
        Command::Validate { model_path, quiet } => {
            // Run the validate command
            let results = validators::validate_models_directory(&model_path);

            let mut error_count = 0;
            let mut success_count = 0;

            for result in &results {
                if result.is_valid {
                    success_count += 1;
                    if !quiet {
                        println!("✅ Valid model structure: {}", result.path.display());
                    }
                } else {
                    error_count += 1;
                    eprintln!("❌ Invalid model structure: {}", result.path.display());
                    for error in &result.errors {
                        eprintln!("   - {}", error);
                    }
                }
            }

            if !quiet || error_count > 0 {
                println!("\nValidation summary:");
                println!("  Valid models: {}", success_count);
                println!("  Invalid models: {}", error_count);
                println!("  Total models checked: {}", results.len());
            }

            if error_count > 0 {
                process::exit(1);
            }
        }
        Command::Version => {
            // Output version information
            println!("FeatherFlow CLI version {}", env!("CARGO_PKG_VERSION"));
            println!("A Rust-based SQL transformation tool similar to dbt");
            println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        }
    }
}
