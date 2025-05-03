use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use sqlparser::dialect::DuckDbDialect;
use walkdir::WalkDir;

use crate::sql_engine::sql_model::{SqlModel, SqlModelCollection};

// ParsedModel has been removed in favor of SqlModel

/// Run the parse command
pub fn parse_command(
    model_path: &PathBuf,
    format: &str,
    validate: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    println!(
        "{}",
        format!("Parsing SQL files in: {}", model_path.display()).green()
    );

    let sql_files = find_sql_files(model_path)?;
    println!("Found {} SQL files", sql_files.len());

    let dialect = DuckDbDialect {};

    let mut model_collection = SqlModelCollection::new();

    let mut success_count = 0;
    let mut structure_warnings = 0;

    for file_path in &sql_files {
        match SqlModel::from_path(file_path, model_path, "duckdb", &dialect) {
            Ok(mut model) => {
                // Check model structure if validation is enabled
                if validate && !model.is_valid_structure {
                    structure_warnings += 1;
                    eprintln!(
                        "{} Invalid file structure for {}: {}",
                        "Warning:".yellow(),
                        file_path.display(),
                        model.structure_errors.join(", ")
                    );
                }

                if let Err(err) = model.extract_dependencies() {
                    eprintln!(
                        "Error extracting dependencies from {}: {}",
                        file_path.display(),
                        err
                    );
                    continue;
                }

                println!("Successfully parsed: {}", file_path.display());
                model_collection.add_model(model);
                success_count += 1;
            }
            Err(err) => {
                eprintln!("Error parsing {}: {}", file_path.display(), err);
            }
        }
    }

    println!(
        "Successfully parsed {} out of {} SQL files in {:.2?}",
        success_count,
        sql_files.len(),
        start_time.elapsed()
    );

    if validate && structure_warnings > 0 {
        println!(
            "{}",
            format!("{} model(s) have file structure issues. Run 'ff validate --model-path {}' for details.",
                structure_warnings,
                model_path.display()
            ).yellow()
        );
    }

    model_collection.build_dependency_graph();

    let cycles = model_collection.detect_cycles();
    if !cycles.is_empty() {
        println!("\n--- {} ---", "Circular Dependencies Detected".red());
        for (i, cycle) in cycles.iter().enumerate() {
            println!("Cycle {}: {}", i + 1, cycle.join(" → "));
        }
    }

    match format {
        "text" => output_text_format(&model_collection),
        "dot" => println!("{}", model_collection.to_dot_graph()),
        "json" => output_json_format(&model_collection)?,
        _ => {
            println!(
                "Unsupported output format: {}. Using text format instead.",
                format
            );
            output_text_format(&model_collection);
        }
    }

    Ok(())
}

/// Output the model collection in text format
fn output_text_format(model_collection: &SqlModelCollection) {
    println!("\n--- {} ---", "Model Dependencies".green());

    match model_collection.get_execution_order() {
        Ok(models) => {
            for model in models {
                println!("\nModel: {}", model.name.bold());

                if !model.referenced_tables.is_empty() {
                    println!("  References:");
                    for table in &model.referenced_tables {
                        println!("    • {}", table);
                    }
                }

                if !model.upstream_models.is_empty() {
                    println!("  Depends on models:");
                    for upstream in &model.upstream_models {
                        println!("    • {}", upstream);
                    }
                }

                if !model.downstream_models.is_empty() {
                    println!("  Used by models:");
                    for downstream in &model.downstream_models {
                        println!("    • {}", downstream);
                    }
                }
            }
        }
        Err(err) => {
            println!("Error determining execution order: {}", err);
        }
    }
}

/// Output the model collection in JSON format
fn output_json_format(
    model_collection: &SqlModelCollection,
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        models: HashMap<String, JsonModel>,
    }

    #[derive(serde::Serialize)]
    struct JsonModel {
        name: String,
        path: String,
        depends_on: Vec<String>,
        referenced_by: Vec<String>,
        referenced_tables: Vec<String>,
    }

    let mut json_models = HashMap::new();

    // Convert models to JSON format
    match model_collection.get_execution_order() {
        Ok(models) => {
            for model in models {
                json_models.insert(
                    model.unique_id.clone(),
                    JsonModel {
                        name: model.name.clone(),
                        path: model.relative_file_path.to_string_lossy().to_string(),
                        depends_on: model.upstream_models.iter().cloned().collect(),
                        referenced_by: model.downstream_models.iter().cloned().collect(),
                        referenced_tables: model.referenced_tables.iter().cloned().collect(),
                    },
                );
            }

            let output = JsonOutput {
                models: json_models,
            };
            let json = serde_json::to_string_pretty(&output)?;
            println!("{}", json);

            Ok(())
        }
        Err(err) => Err(format!("Error determining execution order: {}", err).into()),
    }
}

/// Find all SQL files in the given directory (recursively)
fn find_sql_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut sql_files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "sql" {
                    sql_files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(sql_files)
}
