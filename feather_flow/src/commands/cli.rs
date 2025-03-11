use std::path::PathBuf;
use structopt::StructOpt;

use super::parse;

/// FeatherFlow - A Rust-based alternative to dbt with static SQL analysis
#[derive(StructOpt, Debug)]
#[structopt(name = "featherflow")]
pub enum FeatherFlowCli {
    /// Generate or display the workflow DAG
    #[structopt(name = "dag")]
    Dag {
        #[structopt(short, long)]
        target: String,
    },

    /// Show details about a workflow or task
    #[structopt(name = "show")]
    Show {
        #[structopt(short, long)]
        target: String,
    },

    /// Compile a workflow definition
    #[structopt(name = "compile")]
    Compile {
        #[structopt(short, long)]
        target: String,
    },

    /// Parse and validate SQL models
    #[structopt(name = "parse")]
    Parse {
        /// Path to project configuration file (defaults to ./featherflow_project.yaml)
        #[structopt(short, long)]
        config: Option<PathBuf>,

        /// Target schema to use for table references
        #[structopt(short, long)]
        schema: Option<String>,

        /// Output directory for transformed SQL (optional)
        #[structopt(short, long)]
        output: Option<PathBuf>,

        /// Print detailed information about each model
        #[structopt(short, long)]
        verbose: bool,
    },
}

pub fn parse_cli() -> FeatherFlowCli {
    FeatherFlowCli::from_args()
}

pub fn run_cli() {
    let cli = parse_cli();

    match cli {
        FeatherFlowCli::Dag { target } => {
            println!("Generating DAG for target: {}", target);
        }
        FeatherFlowCli::Show { target } => {
            println!("Showing details for target: {}", target);
        }
        FeatherFlowCli::Compile { target } => {
            println!("Compiling workflow for target: {}", target);
        }
        FeatherFlowCli::Parse {
            config,
            schema,
            output,
            verbose,
        } => {
            println!("Parsing SQL models...");

            match parse::run_parse(config, schema) {
                Ok(models) => {
                    println!("Successfully parsed {} models", models.len());

                    if verbose {
                        println!("\nParsed models:");
                        for (i, model) in models.iter().enumerate() {
                            println!("\n{}. {}", i + 1, model.path.display());
                            println!("   Referenced tables: {:?}", model.referenced_tables);

                            if let Some(output_dir) = &output {
                                // Save transformed SQL to output directory if specified
                                let output_path =
                                    output_dir.join(&model.path).with_extension("parsed.sql");

                                if let Some(parent) = output_path.parent() {
                                    if let Err(e) = std::fs::create_dir_all(parent) {
                                        eprintln!(
                                            "Error creating directory {}: {}",
                                            parent.display(),
                                            e
                                        );
                                        continue;
                                    }
                                }

                                if let Err(e) = std::fs::write(&output_path, &model.transformed_sql)
                                {
                                    eprintln!("Error writing to {}: {}", output_path.display(), e);
                                } else {
                                    println!(
                                        "   Saved transformed SQL to: {}",
                                        output_path.display()
                                    );
                                }
                            }
                        }
                    } else if output.is_some() {
                        // If not verbose but output is specified, still save the files
                        let output_dir = output.unwrap();
                        for model in &models {
                            let output_path =
                                output_dir.join(&model.path).with_extension("parsed.sql");

                            if let Some(parent) = output_path.parent() {
                                if let Err(_) = std::fs::create_dir_all(parent) {
                                    continue;
                                }
                            }

                            let _ = std::fs::write(&output_path, &model.transformed_sql);
                        }
                        println!("Saved transformed SQL files to: {}", output_dir.display());
                    }
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
