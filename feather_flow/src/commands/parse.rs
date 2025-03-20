use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::commands::config::{read_config, FeatherFlowConfig};
use crate::sql_engine::{ast_utils, lineage};

/// File extension for SQL files
const SQL_EXTENSION: &str = "sql";

/// Errors that can occur during parsing
#[derive(Debug)]
pub enum ParseError {
    ConfigError(String),
    IoError(std::io::Error),
    SqlParseError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error)
    }
}

impl From<Box<dyn std::error::Error>> for ParseError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        ParseError::ConfigError(error.to_string())
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            ParseError::IoError(err) => write!(f, "IO error: {}", err),
            ParseError::SqlParseError(msg) => write!(f, "SQL parse error: {}", msg),
        }
    }
}

/// Result type for parse operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Structure representing a parsed SQL model
#[derive(Debug)]
pub struct ParsedModel {
    /// Path to the model file (relative to models directory)
    pub path: PathBuf,
    /// Original SQL content
    pub sql: String,
    /// Modified SQL with schema transformations applied
    pub transformed_sql: String,
    /// Tables referenced in the SQL
    pub referenced_tables: Vec<String>,
    /// Column-level lineage information
    pub column_lineage: Option<Vec<lineage::ColumnLineage>>,
}

/// Runs the parse command
///
/// This function:
/// 1. Reads the project configuration
/// 2. Finds all SQL files in the models directory
/// 3. Parses each SQL file and applies transformations
/// 4. Returns information about the parsed models
pub fn run_parse(
    config_path: Option<PathBuf>,
    target_schema: Option<String>,
) -> ParseResult<Vec<ParsedModel>> {
    // Read the configuration
    let config = read_config(config_path)?;
    println!("Loaded configuration for project: {}", config.name);

    // Get the models directory path
    let models_dir = get_models_path(&config)?;
    println!("Searching for SQL files in: {}", models_dir.display());

    // Find all SQL files in the models directory
    let sql_files = find_sql_files(&models_dir)?;
    let sql_files_count = sql_files.len();
    println!("Found {} SQL files", sql_files_count);

    // Parse and transform each SQL file
    let mut parsed_models = Vec::new();

    for file_path in sql_files {
        match parse_sql_file(&file_path, &models_dir, target_schema.as_deref(), &config) {
            Ok(model) => {
                println!("Successfully parsed: {}", file_path.display());
                parsed_models.push(model);
            }
            Err(err) => {
                eprintln!("Error parsing {}: {}", file_path.display(), err);
                // Continue with the next file instead of failing the entire command
            }
        }
    }

    println!(
        "Successfully parsed {} out of {} SQL files",
        parsed_models.len(),
        sql_files_count
    );

    Ok(parsed_models)
}

/// Gets the absolute path to the models directory
fn get_models_path(config: &FeatherFlowConfig) -> ParseResult<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let models_dir = current_dir.join(&config.models_path);

    if !models_dir.exists() || !models_dir.is_dir() {
        return Err(ParseError::ConfigError(format!(
            "Models directory does not exist: {}",
            models_dir.display()
        )));
    }

    Ok(models_dir)
}

/// Finds all SQL files in the given directory (recursively)
fn find_sql_files(dir: &Path) -> ParseResult<Vec<PathBuf>> {
    let mut sql_files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == SQL_EXTENSION {
                    sql_files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(sql_files)
}

/// Parses a single SQL file and returns metadata about it
fn parse_sql_file(
    file_path: &Path,
    models_dir: &Path,
    target_schema: Option<&str>,
    _config: &FeatherFlowConfig,
) -> ParseResult<ParsedModel> {
    // Read the SQL file
    let sql = fs::read_to_string(file_path)?;

    // Transform the SQL (apply schema changes)
    // Use target_schema if provided, otherwise use "private"
    let _target_schema = target_schema.unwrap_or("private");

    // Use the existing ast_utils to transform the SQL
    // Currently, the target schema is hardcoded in swap_sql_tables
    let transformed_sql = ast_utils::swap_sql_tables(&sql);

    // Get the relative path from the models directory
    let relative_path = file_path.strip_prefix(models_dir).map_err(|_| {
        ParseError::ConfigError(format!(
            "File path is not within models directory: {}",
            file_path.display()
        ))
    })?;

    // Extract referenced tables (this is a simplified implementation)
    // For now, we'll get them from the ast_utils module's output
    let parser = sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::DuckDbDialect {}, &sql)
        .map_err(|e| ParseError::SqlParseError(e.to_string()))?;

    let referenced_tables = ast_utils::get_table_names(&parser);
    
    // Extract column-level lineage
    let column_lineage = match lineage::extract_column_lineage(&sql) {
        Ok(lineage) => Some(lineage),
        Err(err) => {
            eprintln!("Warning: Could not extract column lineage for {}: {}", file_path.display(), err);
            None
        }
    };

    Ok(ParsedModel {
        path: relative_path.to_path_buf(),
        sql,
        transformed_sql,
        referenced_tables,
        column_lineage,
    })
}