use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use sqlparser::dialect::DuckDbDialect;
use walkdir::WalkDir;

use crate::sql_engine::sql_model::{SqlModel, SqlModelCollection};

type ParseResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Parse SQL models and analyze their dependencies
pub fn parse_command(
    model_path: &Path,
    format: &str,
    validate: bool,
    output_file: Option<&str>,
) -> ParseResult<()> {
    let start_time = Instant::now();
    println!(
        "{}",
        format!("Parsing SQL files in: {}", model_path.display()).green()
    );

    // Find SQL files and parse them into models
    let sql_files = find_sql_files(model_path)?;
    println!("Found {} SQL files", sql_files.len());

    let mut model_collection = parse_sql_files(&sql_files, model_path, validate)?;

    // Process the model collection
    process_model_collection(&mut model_collection, model_path, validate)?;

    // Output results in the requested format
    output_results(&model_collection, format, output_file)?;

    println!(
        "Successfully parsed {} out of {} SQL files in {:.2?}",
        model_collection.models_count(),
        sql_files.len(),
        start_time.elapsed()
    );

    Ok(())
}

/// Parse SQL files and build the model collection
fn parse_sql_files(
    sql_files: &[PathBuf],
    model_path: &Path,
    validate: bool,
) -> ParseResult<SqlModelCollection> {
    let dialect = DuckDbDialect {};
    let mut model_collection = SqlModelCollection::new();

    for file_path in sql_files {
        match parse_single_sql_file(file_path.as_path(), model_path, &dialect, validate)? {
            Some(model) => {
                println!("Successfully parsed: {}", file_path.display());
                model_collection.add_model(model);
            }
            None => continue,
        }
    }

    Ok(model_collection)
}

/// Parse a single SQL file into a model
#[allow(clippy::needless_return)]
fn parse_single_sql_file(
    file_path: &Path,
    model_path: &Path,
    dialect: &DuckDbDialect,
    validate: bool,
) -> ParseResult<Option<SqlModel>> {
    // Try to create the model from the file path
    match SqlModel::from_path(file_path, model_path, "duckdb", dialect) {
        Ok(mut model) => {
            // Handle model validation if requested
            if validate {
                validate_model_structure(&model, file_path, model_path)?;
            }

            // Extract model dependencies
            if extract_model_dependencies(&mut model, file_path).is_err() {
                return Ok(None);
            }

            Ok(Some(model))
        }
        Err(err) => handle_model_creation_error(
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Model creation error: {}", err),
            )),
            file_path,
            model_path,
            validate,
        ),
    }
}

/// Validate the structure of a model
fn validate_model_structure(
    model: &SqlModel,
    file_path: &Path,
    model_path: &Path,
) -> ParseResult<()> {
    if !model.is_valid_structure {
        eprintln!(
            "{} Invalid file structure for {}: {}",
            "Error:".red(),
            file_path.display(),
            model.structure_errors.join(", ")
        );

        return Err(format!(
            "Model validation failed. Run 'ff validate --model-path {}' for details.",
            model_path.display()
        )
        .into());
    }

    Ok(())
}

/// Extract dependencies from a model
fn extract_model_dependencies(model: &mut SqlModel, file_path: &Path) -> ParseResult<()> {
    if let Err(err) = model.extract_dependencies() {
        eprintln!(
            "Error extracting dependencies from {}: {}",
            file_path.display(),
            err
        );
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to extract dependencies: {}", err),
        )));
    }

    Ok(())
}

/// Handle errors that occur during model creation
fn handle_model_creation_error(
    err: Box<dyn std::error::Error>,
    file_path: &Path,
    model_path: &Path,
    validate: bool,
) -> ParseResult<Option<SqlModel>> {
    let err_message = err.to_string();

    if err_message.contains("Failed to read SQL file") && validate {
        // Special error case: SQL file missing but YAML exists
        eprintln!(
            "{} Invalid file structure for the directory containing {}: SQL file is missing but YAML file exists",
            "Error:".red(),
            file_path.display()
        );

        Err(format!(
            "Model validation failed. Run 'ff validate --model-path {}' for details.",
            model_path.display()
        )
        .into())
    } else {
        // General parsing error
        eprintln!("Error parsing {}: {}", file_path.display(), err);
        Ok(None)
    }
}

/// Process the model collection (load imports, build dependency graph, validate)
fn process_model_collection(
    model_collection: &mut SqlModelCollection,
    model_path: &Path,
    validate: bool,
) -> ParseResult<()> {
    // Load source definitions from imports directory
    if let Err(err) = model_collection.load_source_definitions(model_path) {
        eprintln!(
            "{} Failed to load source definitions: {}",
            "Warning:".yellow(),
            err
        );
    }

    model_collection.build_dependency_graph();

    // Check for missing imports
    if validate && model_collection.has_missing_sources() {
        println!("\n--- {} ---", "Missing External Imports Detected".red());
        for error in model_collection.get_missing_sources_report() {
            println!("{}", error);
        }
        return Err("Missing external imports detected. Add import definitions to imports directory or check for typos in import references.".into());
    }

    // Detect and report circular dependencies
    let cycles = model_collection.detect_cycles();
    if !cycles.is_empty() {
        println!("\n--- {} ---", "Circular Dependencies Detected".red());
        for (i, cycle) in cycles.iter().enumerate() {
            println!("Cycle {}: {}", i + 1, cycle.join(" → "));
        }
    }

    Ok(())
}

/// Output the results in the requested format
fn output_results(
    model_collection: &SqlModelCollection,
    format: &str,
    output_file: Option<&str>,
) -> ParseResult<()> {
    if let Some(output_path) = output_file {
        // Write to file
        match format {
            "yaml" => write_yaml_to_file(model_collection, output_path)?,
            _ => {
                println!(
                    "When using --output-file, only 'yaml' format is supported. Using yaml format."
                );
                write_yaml_to_file(model_collection, output_path)?;
            }
        }
    } else {
        // Output to stdout
        match format {
            "text" => output_text_format(model_collection),
            "dot" => println!("{}", model_collection.to_dot_graph()),
            "json" => output_json_format(model_collection)?,
            "yaml" => output_yaml_format(model_collection)?,
            _ => {
                println!(
                    "Unsupported output format: {}. Using text format instead.",
                    format
                );
                output_text_format(model_collection);
            }
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
                print_model_summary(model);
                print_model_details(model);
                print_model_dependencies(model);
            }
        }
        Err(err) => {
            println!("Error determining execution order: {}", err);
        }
    }
}

/// Print basic model information including name and depth
fn print_model_summary(model: &SqlModel) {
    let depth_info = match model.depth {
        Some(depth) => format!(" [Depth: {}]", depth),
        None => " [Depth: unknown]".to_string(),
    };
    println!("\nModel: {}{}", model.name.bold(), depth_info.yellow());
}

/// Print detailed model metadata (description, materialization, location, tags, columns)
fn print_model_details(model: &SqlModel) {
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

    print_model_columns(model);
}

/// Print model column information
fn print_model_columns(model: &SqlModel) {
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
}

/// Print model dependency information (external sources, upstream and downstream models)
fn print_model_dependencies(model: &SqlModel) {
    // Print external sources
    let external_sources = model.get_external_sources();
    if !external_sources.is_empty() {
        println!("  External sources:");
        for source in &external_sources {
            println!("    • {}", source);
        }
    }

    // Print upstream dependencies
    if !model.upstream_models.is_empty() {
        println!("  Depends on models:");
        for upstream in &model.upstream_models {
            println!("    • {}", upstream);
        }
    }

    // Print downstream dependencies
    if !model.downstream_models.is_empty() {
        println!("  Used by models:");
        for downstream in &model.downstream_models {
            println!("    • {}", downstream);
        }
    }
}

/// Output the model collection in JSON format
fn output_json_format(model_collection: &SqlModelCollection) -> ParseResult<()> {
    #[allow(dead_code)]
    #[derive(serde::Serialize)]
    struct JsonOutput {
        models: HashMap<String, JsonModel>,
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    #[derive(serde::Serialize)]
    struct JsonColumn {
        name: String,
        description: Option<String>,
        data_type: Option<String>,
    }

    let json_models = build_json_models(model_collection)?;

    let output = output_json_format::JsonOutput {
        models: json_models,
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    Ok(())
}

/// Convert the model collection to a JSON models map
fn build_json_models(
    model_collection: &SqlModelCollection,
) -> ParseResult<HashMap<String, output_json_format::JsonModel>> {
    let mut json_models = HashMap::new();

    match model_collection.get_execution_order() {
        Ok(models) => {
            for model in models {
                // Convert model to JSON representation
                json_models.insert(model.unique_id.clone(), convert_model_to_json(model));
            }
            Ok(json_models)
        }
        Err(err) => Err(format!("Error determining execution order: {}", err).into()),
    }
}

/// JSON output format definitions
mod output_json_format {
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize)]
    pub struct JsonOutput {
        pub models: HashMap<String, JsonModel>,
    }

    #[derive(Serialize)]
    pub struct JsonModel {
        pub name: String,
        pub path: String,
        pub description: Option<String>,
        pub materialized: Option<String>,
        pub database: Option<String>,
        pub schema: Option<String>,
        pub object_name: Option<String>,
        pub tags: Vec<String>,
        pub columns: Vec<JsonColumn>,
        pub depends_on: Vec<String>,
        pub referenced_by: Vec<String>,
        pub external_sources: Vec<String>,
        pub depth: Option<usize>,
    }

    #[derive(Serialize)]
    pub struct JsonColumn {
        pub name: String,
        pub description: Option<String>,
        pub data_type: Option<String>,
    }
}

/// Convert a single SqlModel to its JSON representation
fn convert_model_to_json(model: &SqlModel) -> output_json_format::JsonModel {
    // Convert column information
    let columns: Vec<output_json_format::JsonColumn> = model
        .columns
        .values()
        .map(|col| output_json_format::JsonColumn {
            name: col.name.clone(),
            description: col.description.clone(),
            data_type: col.data_type.clone(),
        })
        .collect();

    let external_sources: Vec<String> = model.get_external_sources().into_iter().collect();

    output_json_format::JsonModel {
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
    }
}

/// Output the model collection in YAML format
fn output_yaml_format(model_collection: &SqlModelCollection) -> ParseResult<()> {
    let yaml = generate_yaml(model_collection)?;
    println!("{}", yaml);
    Ok(())
}

/// Write the model collection to a YAML file
fn write_yaml_to_file(model_collection: &SqlModelCollection, file_path: &str) -> ParseResult<()> {
    let yaml = generate_yaml(model_collection)?;
    std::fs::write(file_path, yaml)?;
    println!("Model graph data written to {}", file_path);
    Ok(())
}

/// Generate YAML string from model collection
fn generate_yaml(model_collection: &SqlModelCollection) -> ParseResult<String> {
    match model_collection.to_yaml() {
        Ok(yaml_output) => {
            let yaml = serde_yaml::to_string(&yaml_output)?;
            Ok(yaml)
        }
        Err(err) => Err(format!("Error generating YAML: {}", err).into()),
    }
}

/// Helper function to check if a path is in the imports directory structure
fn is_imports_directory(path: &Path) -> bool {
    // Check if the path contains "/imports/" in its string representation
    let has_imports_in_path = path.to_string_lossy().contains("/imports/");

    // Check if the path's filename is "imports"
    let is_imports_dir = path
        .file_name()
        .is_some_and(|name| name.to_string_lossy() == "imports");

    // Return true if either condition is met
    has_imports_in_path || is_imports_dir
}

/// Find all SQL files in the given directory (recursively)
fn find_sql_files(dir: &Path) -> ParseResult<Vec<PathBuf>> {
    // Find actual SQL files
    let mut sql_files = find_actual_sql_files(dir);

    // Find missing SQL files (where YML exists but SQL doesn't)
    // Only check for missing files if we found at least one actual SQL file
    if !sql_files.is_empty() {
        let missing_sql_files = find_missing_sql_files(dir);
        sql_files.extend(missing_sql_files);
    }

    Ok(sql_files)
}

/// Find actual SQL files in the directory
fn find_actual_sql_files(dir: &Path) -> Vec<PathBuf> {
    let mut sql_files = Vec::new();

    // Walk through the directory tree
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        // Check if the file has a .sql extension
        if is_sql_file(path) {
            sql_files.push(path.to_path_buf());
        }
    }

    sql_files
}

/// Check if a path points to a SQL file
fn is_sql_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "sql")
}

/// Check if a path points to a YAML file
fn is_yaml_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "yml")
}

/// Find directories with YAML files but missing SQL files
fn find_missing_sql_files(dir: &Path) -> Vec<PathBuf> {
    let mut missing_sql_files = Vec::new();
    let yaml_only_dirs = find_yaml_only_directories(dir);

    for yaml_dir in yaml_only_dirs {
        // Skip imports directories
        if is_imports_directory(&yaml_dir) {
            continue;
        }

        // Create the expected SQL file path based on the directory name
        if let Some(expected_sql_path) = create_expected_sql_path(&yaml_dir) {
            missing_sql_files.push(expected_sql_path);
        }
    }

    missing_sql_files
}

/// Create the expected SQL file path from a directory
fn create_expected_sql_path(dir: &Path) -> Option<PathBuf> {
    dir.file_name().map(|dir_name| {
        let expected_sql_name = format!("{}.sql", dir_name.to_string_lossy());
        dir.join(&expected_sql_name)
    })
}

/// Find directories that have YAML files but no corresponding SQL files
fn find_yaml_only_directories(dir: &Path) -> Vec<PathBuf> {
    let mut yaml_only_dirs = Vec::new();

    // Walk through the directory tree
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        // Skip files that aren't YAML
        if !is_yaml_file(path) {
            continue;
        }

        // Process YAML file to check for missing SQL
        process_yaml_file_for_missing_sql(path, &mut yaml_only_dirs);
    }

    // Deduplicate directories
    deduplicate_paths(&mut yaml_only_dirs);

    yaml_only_dirs
}

/// Process a YAML file to check if it has a corresponding SQL file
fn process_yaml_file_for_missing_sql(yaml_path: &Path, yaml_only_dirs: &mut Vec<PathBuf>) {
    if let Some(file_stem) = yaml_path.file_stem() {
        if let Some(parent_dir) = yaml_path.parent() {
            // Skip imports directories - they are allowed to have only YAML files
            if is_imports_directory(parent_dir) {
                return;
            }

            // Check if the corresponding SQL file exists
            let expected_sql_file = parent_dir.join(format!("{}.sql", file_stem.to_string_lossy()));
            if !expected_sql_file.exists() {
                yaml_only_dirs.push(parent_dir.to_path_buf());
            }
        }
    }
}

/// Deduplicate a vector of paths
fn deduplicate_paths(paths: &mut Vec<PathBuf>) {
    paths.sort();
    paths.dedup();
}
