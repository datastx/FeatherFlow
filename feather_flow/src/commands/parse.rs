use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use sqlparser::dialect::DuckDbDialect;
use walkdir::WalkDir;

use crate::sql_engine::sql_model::{SqlModel, SqlModelCollection};

type ParseResult<T> = Result<T, Box<dyn std::error::Error>>;

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

    let sql_files = find_sql_files(model_path)?;
    println!("Found {} SQL files", sql_files.len());

    let mut model_collection = parse_sql_files(&sql_files, model_path, validate)?;
    process_model_collection(&mut model_collection, model_path, validate)?;
    output_results(&model_collection, format, output_file)?;

    println!(
        "Successfully parsed {} out of {} SQL files in {:.2?}",
        model_collection.models_count(),
        sql_files.len(),
        start_time.elapsed()
    );

    Ok(())
}

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

#[allow(clippy::needless_return)]
fn parse_single_sql_file(
    file_path: &Path,
    model_path: &Path,
    dialect: &DuckDbDialect,
    validate: bool,
) -> ParseResult<Option<SqlModel>> {
    match SqlModel::from_path(file_path, model_path, "duckdb", dialect) {
        Ok(mut model) => {
            if validate {
                validate_model_structure(&model, file_path, model_path)?;
            }

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

fn handle_model_creation_error(
    err: Box<dyn std::error::Error>,
    file_path: &Path,
    model_path: &Path,
    validate: bool,
) -> ParseResult<Option<SqlModel>> {
    let err_message = err.to_string();

    if err_message.contains("Failed to read SQL file") && validate {
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
        eprintln!("Error parsing {}: {}", file_path.display(), err);
        Ok(None)
    }
}

fn process_model_collection(
    model_collection: &mut SqlModelCollection,
    model_path: &Path,
    validate: bool,
) -> ParseResult<()> {
    if let Err(err) = model_collection.load_source_definitions(model_path) {
        eprintln!(
            "{} Failed to load source definitions: {}",
            "Warning:".yellow(),
            err
        );
    }

    model_collection.build_dependency_graph();

    if validate && model_collection.has_missing_sources() {
        println!("\n--- {} ---", "Missing External Imports Detected".red());
        for error in model_collection.get_missing_sources_report() {
            println!("{}", error);
        }
        return Err("Missing external imports detected. Add import definitions to imports directory or check for typos in import references.".into());
    }

    let cycles = model_collection.detect_cycles();
    if !cycles.is_empty() {
        println!("\n--- {} ---", "Circular Dependencies Detected".red());
        for (i, cycle) in cycles.iter().enumerate() {
            println!("Cycle {}: {}", i + 1, cycle.join(" → "));
        }
    }

    Ok(())
}

fn output_results(
    model_collection: &SqlModelCollection,
    format: &str,
    output_file: Option<&str>,
) -> ParseResult<()> {
    if let Some(output_path) = output_file {
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

fn print_model_summary(model: &SqlModel) {
    let depth_info = match model.depth {
        Some(depth) => format!(" [Depth: {}]", depth),
        None => " [Depth: unknown]".to_string(),
    };
    println!("\nModel: {}{}", model.name.bold(), depth_info.yellow());
}

fn print_model_details(model: &SqlModel) {
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

fn print_model_dependencies(model: &SqlModel) {
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

fn build_json_models(
    model_collection: &SqlModelCollection,
) -> ParseResult<HashMap<String, output_json_format::JsonModel>> {
    let mut json_models = HashMap::new();

    match model_collection.get_execution_order() {
        Ok(models) => {
            for model in models {
                json_models.insert(model.unique_id.clone(), convert_model_to_json(model));
            }
            Ok(json_models)
        }
        Err(err) => Err(format!("Error determining execution order: {}", err).into()),
    }
}

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

fn convert_model_to_json(model: &SqlModel) -> output_json_format::JsonModel {
    let columns: Vec<output_json_format::JsonColumn> = model
        .columns
        .values()
        .map(|col| output_json_format::JsonColumn {
            name: col.name.clone(),
            description: col.description.clone(),
            data_type: col.data_type.clone(),
        })
        .collect();

    let mut external_sources: Vec<String> = model.get_external_sources().into_iter().collect();
    external_sources.sort();

    let mut depends_on: Vec<String> = model.upstream_models.iter().cloned().collect();
    depends_on.sort();

    let mut referenced_by: Vec<String> = model.downstream_models.iter().cloned().collect();
    referenced_by.sort();

    let mut tags = model.tags.clone();
    tags.sort();

    output_json_format::JsonModel {
        name: model.name.clone(),
        path: model.relative_file_path.to_string_lossy().to_string(),
        description: model.description.clone(),
        materialized: model.materialized.clone(),
        database: model.database.clone(),
        schema: model.schema.clone(),
        object_name: model.object_name.clone(),
        tags,
        columns,
        depends_on,
        referenced_by,
        external_sources,
        depth: model.depth,
    }
}

fn output_yaml_format(model_collection: &SqlModelCollection) -> ParseResult<()> {
    let yaml = generate_yaml(model_collection)?;
    println!("{}", yaml);
    Ok(())
}

fn write_yaml_to_file(model_collection: &SqlModelCollection, file_path: &str) -> ParseResult<()> {
    let yaml = generate_yaml(model_collection)?;
    std::fs::write(file_path, yaml)?;
    println!("Model graph data written to {}", file_path);
    Ok(())
}

fn generate_yaml(model_collection: &SqlModelCollection) -> ParseResult<String> {
    match model_collection.to_yaml() {
        Ok(yaml_output) => {
            let yaml = serde_yaml::to_string(&yaml_output)?;
            Ok(yaml)
        }
        Err(err) => Err(format!("Error generating YAML: {}", err).into()),
    }
}

fn is_imports_directory(path: &Path) -> bool {
    let has_imports_in_path = path.to_string_lossy().contains("/imports/");
    let is_imports_dir = path
        .file_name()
        .is_some_and(|name| name.to_string_lossy() == "imports");

    has_imports_in_path || is_imports_dir
}

fn find_sql_files(dir: &Path) -> ParseResult<Vec<PathBuf>> {
    let mut sql_files = find_actual_sql_files(dir);

    if !sql_files.is_empty() {
        let missing_sql_files = find_missing_sql_files(dir);
        sql_files.extend(missing_sql_files);
    }

    sql_files.sort();
    Ok(sql_files)
}

fn find_actual_sql_files(dir: &Path) -> Vec<PathBuf> {
    let mut sql_files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if is_sql_file(path) {
            sql_files.push(path.to_path_buf());
        }
    }

    sql_files
}

fn is_sql_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "sql")
}

fn is_yaml_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "yml")
}

fn find_missing_sql_files(dir: &Path) -> Vec<PathBuf> {
    let mut missing_sql_files = Vec::new();
    let yaml_only_dirs = find_yaml_only_directories(dir);

    for yaml_dir in yaml_only_dirs {
        if is_imports_directory(&yaml_dir) {
            continue;
        }

        if let Some(expected_sql_path) = create_expected_sql_path(&yaml_dir) {
            missing_sql_files.push(expected_sql_path);
        }
    }

    missing_sql_files
}

fn create_expected_sql_path(dir: &Path) -> Option<PathBuf> {
    dir.file_name().map(|dir_name| {
        let expected_sql_name = format!("{}.sql", dir_name.to_string_lossy());
        dir.join(&expected_sql_name)
    })
}

fn find_yaml_only_directories(dir: &Path) -> Vec<PathBuf> {
    let mut yaml_only_dirs = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !is_yaml_file(path) {
            continue;
        }

        process_yaml_file_for_missing_sql(path, &mut yaml_only_dirs);
    }

    deduplicate_paths(&mut yaml_only_dirs);
    yaml_only_dirs
}

fn process_yaml_file_for_missing_sql(yaml_path: &Path, yaml_only_dirs: &mut Vec<PathBuf>) {
    if let Some(file_stem) = yaml_path.file_stem() {
        if let Some(parent_dir) = yaml_path.parent() {
            if is_imports_directory(parent_dir) {
                return;
            }

            let expected_sql_file = parent_dir.join(format!("{}.sql", file_stem.to_string_lossy()));
            if !expected_sql_file.exists() {
                yaml_only_dirs.push(parent_dir.to_path_buf());
            }
        }
    }
}

fn deduplicate_paths(paths: &mut Vec<PathBuf>) {
    paths.sort();
    paths.dedup();
}
