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

    for file_path in &sql_files {
        match SqlModel::from_path(file_path, model_path, "duckdb", &dialect) {
            Ok(mut model) => {
                // Check model structure if validation is enabled - Error out if invalid
                if validate && !model.is_valid_structure {
                    eprintln!(
                        "{} Invalid file structure for {}: {}",
                        "Error:".red(),
                        file_path.display(),
                        model.structure_errors.join(", ")
                    );
                    // Stop processing and return an error
                    return Err(format!(
                        "Model validation failed. Run 'ff validate --model-path {}' for details.",
                        model_path.display()
                    )
                    .into());
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
                // Check if this is a "no such file" error
                if err.to_string().contains("Failed to read SQL file") && validate {
                    // If we're validating and the SQL file doesn't exist (but YAML might), show a validation error
                    eprintln!(
                        "{} Invalid file structure for the directory containing {}: SQL file is missing but YAML file exists",
                        "Error:".red(),
                        file_path.display()
                    );
                    return Err(format!(
                        "Model validation failed. Run 'ff validate --model-path {}' for details.",
                        model_path.display()
                    )
                    .into());
                } else {
                    eprintln!("Error parsing {}: {}", file_path.display(), err);
                }
            }
        }
    }

    println!(
        "Successfully parsed {} out of {} SQL files in {:.2?}",
        success_count,
        sql_files.len(),
        start_time.elapsed()
    );

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
                // Display model name with depth information
                let depth_info = match model.depth {
                    Some(depth) => format!(" [Depth: {}]", depth),
                    None => " [Depth: unknown]".to_string(),
                };
                println!("\nModel: {}{}", model.name.bold(), depth_info.yellow());

                // Print model metadata from YAML
                if let Some(ref description) = model.description {
                    println!("  Description: {}", description);
                }

                if let Some(ref materialized) = model.materialized {
                    println!("  Materialized: {}", materialized);
                }

                if let Some(ref schema) = model.schema {
                    let db = model.database.as_deref().unwrap_or("default");
                    println!(
                        "  Location: {}.{}.{}",
                        db,
                        schema,
                        model.object_name.as_deref().unwrap_or(&model.name)
                    );
                }

                if !model.tags.is_empty() {
                    println!("  Tags: {}", model.tags.join(", "));
                }

                // Show column information if available
                if !model.columns.is_empty() {
                    println!("  Columns:");
                    for column in model.columns.values() {
                        let data_type = column.data_type.as_deref().unwrap_or("unknown");
                        print!("    • {} [{}]", column.name, data_type);

                        if let Some(ref desc) = column.description {
                            print!(": {}", desc);
                        }
                        println!();
                    }
                }

                // Print dependencies
                // Get external sources (tables that are not models in the project)
                let external_sources = model.get_external_sources();
                if !external_sources.is_empty() {
                    println!("  External sources:");
                    for source in &external_sources {
                        println!("    • {}", source);
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
        description: Option<String>,
        materialized: Option<String>,
        database: Option<String>,
        schema: Option<String>,
        object_name: Option<String>,
        tags: Vec<String>,
        columns: Vec<JsonColumn>,
        depends_on: Vec<String>,
        referenced_by: Vec<String>,
        external_sources: Vec<String>,
        depth: Option<usize>,
    }

    #[derive(serde::Serialize)]
    struct JsonColumn {
        name: String,
        description: Option<String>,
        data_type: Option<String>,
    }

    let mut json_models = HashMap::new();

    // Convert models to JSON format
    match model_collection.get_execution_order() {
        Ok(models) => {
            for model in models {
                // Convert column information
                let columns: Vec<JsonColumn> = model
                    .columns
                    .values()
                    .map(|col| JsonColumn {
                        name: col.name.clone(),
                        description: col.description.clone(),
                        data_type: col.data_type.clone(),
                    })
                    .collect();

                let external_sources: Vec<String> =
                    model.get_external_sources().into_iter().collect();

                json_models.insert(
                    model.unique_id.clone(),
                    JsonModel {
                        name: model.name.clone(),
                        path: model.relative_file_path.to_string_lossy().to_string(),
                        description: model.description.clone(),
                        materialized: model.materialized.clone(),
                        database: model.database.clone(),
                        schema: model.schema.clone(),
                        object_name: model.object_name.clone(),
                        tags: model.tags.clone(),
                        columns,
                        depends_on: model.upstream_models.iter().cloned().collect(),
                        referenced_by: model.downstream_models.iter().cloned().collect(),
                        external_sources,
                        depth: model.depth,
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

    // If we're not running validation separately, let's check for YAML files without SQL files
    // This helps catch cases where a YAML file exists but the SQL file was deleted
    if !sql_files.is_empty() {
        // For each SQL file, get its parent directory and check if there's a corresponding YAML file
        let mut unexpected_yaml_dirs = Vec::new();

        // Find directories that may have a YAML but no SQL
        for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "yml" {
                        if let Some(file_stem) = path.file_stem() {
                            if let Some(parent_dir) = path.parent() {
                                let expected_sql_file =
                                    parent_dir.join(format!("{}.sql", file_stem.to_string_lossy()));
                                if !expected_sql_file.exists() {
                                    unexpected_yaml_dirs.push(parent_dir.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add these directories to the validation results
        if !unexpected_yaml_dirs.is_empty() {
            // Deduplicate directories
            unexpected_yaml_dirs.sort();
            unexpected_yaml_dirs.dedup();

            for dir in unexpected_yaml_dirs {
                // We just need to add the expected SQL file to the sql_files list
                // when validation runs it will find that it's missing
                if let Some(dir_name) = dir.file_name() {
                    let expected_sql_name = format!("{}.sql", dir_name.to_string_lossy());
                    let expected_sql_path = dir.join(&expected_sql_name);
                    sql_files.push(expected_sql_path);
                }
            }
        }
    }

    Ok(sql_files)
}
