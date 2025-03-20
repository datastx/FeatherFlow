use std::path::PathBuf;
use std::process;
use clap::{Parser};

use crate::commands::parse::run_parse;
use crate::sql_engine::lineage::generate_lineage_graph;

/// FeatherFlow CLI arguments
#[derive(Parser, Debug)]
#[clap(name = "featherflow")]
pub enum FeatherFlowCli {
    /// Generate a DAG visualization
    #[clap(name = "dag")]
    Dag {
        /// Path to the configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },

    /// Show model information
    #[clap(name = "show")]
    Show {
        /// Path to the configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },

    /// Compile models
    #[clap(name = "compile")]
    Compile {
        /// Path to the configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },

    /// Parse SQL models
    #[clap(name = "parse")]
    Parse {
        /// Path to the configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,

        /// Target schema to use
        #[clap(short, long)]
        schema: Option<String>,

        /// Output transformed SQL
        #[clap(short, long)]
        output: bool,
        
        /// Extract column-level lineage
        #[clap(short, long)]
        lineage: bool,
    },
}

/// Get the CLI arguments
pub fn get_cli_args() -> FeatherFlowCli {
    FeatherFlowCli::parse()
}

/// Run the CLI command
pub fn run_cli() {
    match get_cli_args() {
        FeatherFlowCli::Parse {
            config,
            schema,
            output,
            lineage,
        } => {
            // Run the parse command
            match run_parse(config, schema) {
                Ok(models) => {
                    println!("Successfully parsed {} models", models.len());

                    // Output the transformed SQL for each model if requested
                    if output {
                        for model in &models {
                            println!("\n--- {} ---", model.path.display());
                            println!("{}", model.transformed_sql);
                        }
                    }

                    // Output lineage information if requested
                    if lineage {
                        for model in &models {
                            println!("\n--- Lineage for {} ---", model.path.display());
                            if let Some(lineage_info) = &model.column_lineage {
                                if lineage_info.is_empty() {
                                    println!("No column lineage information available.");
                                } else {
                                    // Output in dot format (which can be used with Graphviz)
                                    let graph = generate_lineage_graph(lineage_info);
                                    println!("{}", graph);
                                }
                            } else {
                                println!("No column lineage information available.");
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        }
        FeatherFlowCli::Dag { .. } => {
            println!("DAG generation is not yet implemented");
        }
        FeatherFlowCli::Show { .. } => {
            println!("Show command is not yet implemented");
        }
        FeatherFlowCli::Compile { .. } => {
            println!("Compile command is not yet implemented");
        }
    }
}