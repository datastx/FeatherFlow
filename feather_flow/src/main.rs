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
    }
}
