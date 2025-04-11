use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use crate::commands::parse::{run_parse, ParsedModel};
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

        /// Generate dependency graph
        #[clap(short = 'g', long)]
        graph: bool,

        /// Output format for graph (dot, json, text)
        #[clap(long, default_value = "dot")]
        format: String,
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
            graph,
            format,
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

                    // Generate dependency graph if requested
                    if graph {
                        println!("\n--- Generating Model Dependency Graph ---");
                        let model_graph = build_dependency_graph(&models);

                        // Output the graph in the requested format
                        match format.as_str() {
                            "dot" => println!("{}", generate_dot_graph(&model_graph)),
                            "json" => println!("{}", generate_json_graph(&model_graph)),
                            "text" => print_text_graph(&model_graph),
                            _ => eprintln!("Unsupported graph format: {}", format),
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

/// Build a dependency graph from parsed models
fn build_dependency_graph(models: &[ParsedModel]) -> HashMap<String, Vec<String>> {
    // Map model names to their file paths for easy lookup
    let mut model_map: HashMap<String, &ParsedModel> = HashMap::new();

    for model in models {
        // Extract model name from path (without extension)
        if let Some(file_name) = model.path.file_stem() {
            if let Some(name) = file_name.to_str() {
                model_map.insert(name.to_string(), model);
            }
        }
    }

    // Build dependency graph (model -> models it depends on)
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for (model_name, model) in &model_map {
        let mut dependencies = Vec::new();

        for table in &model.referenced_tables {
            // Extract table name (without schema)
            let table_parts: Vec<&str> = table.split('.').collect();
            let table_name = if table_parts.len() > 1 {
                table_parts[1]
            } else {
                table_parts[0]
            };

            // Check if the referenced table matches another model
            if model_map.contains_key(table_name) && table_name != model_name {
                dependencies.push(table_name.to_string());
            }
        }

        // Only add to graph if it has dependencies
        if !dependencies.is_empty() {
            graph.insert(model_name.clone(), dependencies);
        } else {
            // Add model with empty dependencies
            graph.insert(model_name.clone(), Vec::new());
        }
    }

    graph
}

/// Generate a DOT graph representation
fn generate_dot_graph(graph: &HashMap<String, Vec<String>>) -> String {
    let mut dot = String::from("digraph G {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n\n");

    // Add nodes for all models
    for model in graph.keys() {
        dot.push_str(&format!("  \"{}\";\n", model));
    }

    dot.push_str("\n");

    // Add edges for dependencies
    for (model, dependencies) in graph {
        for dep in dependencies {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", dep, model));
        }
    }

    dot.push_str("}\n");
    dot
}

/// Generate a JSON graph representation
fn generate_json_graph(graph: &HashMap<String, Vec<String>>) -> String {
    use std::collections::BTreeMap;

    // Convert to BTreeMap for consistent ordering
    let ordered_graph: BTreeMap<&String, &Vec<String>> = graph.iter().collect();

    // Simple JSON serialization
    let mut json = String::from("{\n");

    for (i, (model, dependencies)) in ordered_graph.iter().enumerate() {
        json.push_str(&format!("  \"{}\": [", model));

        for (j, dep) in dependencies.iter().enumerate() {
            if j > 0 {
                json.push_str(", ");
            }
            json.push_str(&format!("\"{}\"", dep));
        }

        json.push_str("]");

        if i < ordered_graph.len() - 1 {
            json.push_str(",\n");
        } else {
            json.push_str("\n");
        }
    }

    json.push_str("}\n");
    json
}

/// Print a text representation of the graph
fn print_text_graph(graph: &HashMap<String, Vec<String>>) {
    use std::collections::BTreeMap;

    // Convert to BTreeMap for consistent ordering
    let ordered_graph: BTreeMap<&String, &Vec<String>> = graph.iter().collect();

    println!("Model Dependency Graph:");
    println!("======================");

    for (model, dependencies) in ordered_graph {
        println!("Model: {}", model);

        if dependencies.is_empty() {
            println!("  No dependencies");
        } else {
            println!("  Depends on:");
            for dep in dependencies {
                println!("    - {}", dep);
            }
        }
        println!();
    }
}
