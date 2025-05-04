//! Validation utilities for FeatherFlow

use std::fs;
use std::path::{Path, PathBuf};

/// Result of a file structure validation
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Whether the validation passed or failed
    pub is_valid: bool,
    /// Path that was validated
    pub path: PathBuf,
    /// List of validation errors
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Create a new valid result
    pub fn valid(path: PathBuf) -> Self {
        Self {
            is_valid: true,
            path,
            errors: Vec::new(),
        }
    }

    /// Create a new invalid result with errors
    pub fn invalid(path: PathBuf, errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            path,
            errors,
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
}

/// Validates that a model follows the proper file structure:
/// - Each model has its own directory with the same name as the model (without extension)
/// - The directory contains exactly one .sql file with the same name as the directory
/// - The directory contains exactly one .yml file with the same name as the directory
/// - Special case: 'imports' directory and its subdirectories only require .yml files
///
/// Example: models/staging/stg_customers/stg_customers.sql and models/staging/stg_customers/stg_customers.yml
pub fn validate_model_structure(path: &Path) -> ValidationResult {
    let mut result = ValidationResult::valid(path.to_path_buf());

    // Check that the path is a directory
    if !path.is_dir() {
        result.add_error(format!("Path is not a directory: {}", path.display()));
        return result;
    }

    // Get the directory name
    let dir_name = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            result.add_error(format!(
                "Could not get directory name for: {}",
                path.display()
            ));
            return result;
        }
    };

    // Check if this is in the imports directory structure
    let is_imports = path.to_string_lossy().contains("/imports/")
        || path.to_string_lossy().ends_with("/imports");

    // For regular models (not in imports directory), require SQL file
    // Skip this check for imports directory since they may only have YAML files
    if !is_imports {
        // Check if we have a SQL file matching the directory name
        let sql_file_path = path.join(format!("{}.sql", dir_name));
        if !sql_file_path.exists() {
            result.add_error(format!(
                "Missing SQL file: {} (expected at {})",
                dir_name,
                sql_file_path.display()
            ));
        }
    }

    // Check if we have a YAML file matching the directory name
    let yaml_file_path = path.join(format!("{}.yml", dir_name));
    if !yaml_file_path.exists() {
        result.add_error(format!(
            "Missing YAML file: {} (expected at {})",
            dir_name,
            yaml_file_path.display()
        ));
    }

    // Check for other unexpected files
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            result.add_error(format!(
                "Failed to read directory {}: {}",
                path.display(),
                e
            ));
            return result;
        }
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() {
            let file_name = entry_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Allow only the expected SQL (if not imports) and YAML files
            let expected_sql = format!("{}.sql", dir_name);
            let expected_yml = format!("{}.yml", dir_name);

            // Check if this is a valid file for this directory
            let is_valid_file =
                file_name == expected_yml || (!is_imports && file_name == expected_sql);

            if !is_valid_file {
                if is_imports {
                    result.add_error(format!(
                        "Unexpected file in imports directory: {} (only {} is expected)",
                        file_name, expected_yml
                    ));
                } else {
                    result.add_error(format!(
                        "Unexpected file in model directory: {} (only {} and {} are expected)",
                        file_name, expected_sql, expected_yml
                    ));
                }
            }
        }
    }

    result
}

/// Validates a directory of models to ensure each follows the proper file structure
pub fn validate_models_directory(models_dir: &Path) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // Check that the path is a directory
    if !models_dir.is_dir() {
        let result = ValidationResult::invalid(
            models_dir.to_path_buf(),
            vec![format!("Path is not a directory: {}", models_dir.display())],
        );
        results.push(result);
        return results;
    }

    // Collect all model directories recursively
    collect_model_directories(models_dir, &mut results);

    results
}

/// Helper function to check if a path is in the imports directory structure
fn is_imports_directory(path: &Path) -> bool {
    path.to_string_lossy().contains("/imports/")
        || path
            .file_name()
            .map(|n| n.to_string_lossy() == "imports")
            .unwrap_or(false)
}

/// Recursively collects and validates model directories
fn collect_model_directories(dir: &Path, results: &mut Vec<ValidationResult>) {
    if !dir.is_dir() {
        return;
    }

    // Check if this directory is in the imports path
    let is_imports = is_imports_directory(dir);

    // Handle imports directory structure specially
    if is_imports {
        // For imports directories, we consider them valid if they have the matching YAML file
        // We don't require SQL files in imports directories
        let dir_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let yml_file = dir.join(format!("{}.yml", dir_name));

        // Special case for imports directories
        if yml_file.exists() {
            // Add a successful validation result for this imports directory
            let result = ValidationResult::valid(dir.to_path_buf());
            results.push(result);
        }

        // Recursively process subdirectories of imports
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_model_directories(&path, results);
                }
            }
        }

        return; // Skip regular model directory validation for imports
    }

    // Handle regular model directories

    // Read directory entries
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    // Determine if this is a model directory by checking for matching SQL or YAML files
    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let sql_file = dir.join(format!("{}.sql", dir_name));
    let yml_file = dir.join(format!("{}.yml", dir_name));

    let is_model_dir = sql_file.exists() || yml_file.exists();

    // If it looks like a model directory, validate it
    if is_model_dir {
        results.push(validate_model_structure(dir));
    }

    // Process subdirectories regardless of whether this is a model directory
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_model_directories(&path, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_validate_model_structure_valid() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().join("test_model");
        fs::create_dir(&model_dir).unwrap();

        // Create valid model files
        let sql_file = model_dir.join("test_model.sql");
        let yml_file = model_dir.join("test_model.yml");

        File::create(&sql_file)
            .unwrap()
            .write_all(b"SELECT * FROM table")
            .unwrap();
        File::create(&yml_file)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        let result = validate_model_structure(&model_dir);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_model_structure_missing_sql() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().join("test_model");
        fs::create_dir(&model_dir).unwrap();

        // Create only YAML file
        let yml_file = model_dir.join("test_model.yml");
        File::create(&yml_file)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        let result = validate_model_structure(&model_dir);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("Missing SQL file"));
    }

    #[test]
    fn test_validate_model_structure_missing_yml() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().join("test_model");
        fs::create_dir(&model_dir).unwrap();

        // Create only SQL file
        let sql_file = model_dir.join("test_model.sql");
        File::create(&sql_file)
            .unwrap()
            .write_all(b"SELECT * FROM table")
            .unwrap();

        let result = validate_model_structure(&model_dir);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("Missing YAML file"));
    }

    #[test]
    fn test_validate_model_structure_unexpected_file() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().join("test_model");
        fs::create_dir(&model_dir).unwrap();

        // Create valid model files
        let sql_file = model_dir.join("test_model.sql");
        let yml_file = model_dir.join("test_model.yml");

        File::create(&sql_file)
            .unwrap()
            .write_all(b"SELECT * FROM table")
            .unwrap();
        File::create(&yml_file)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        // Create unexpected file
        let unexpected_file = model_dir.join("unexpected.md");
        File::create(&unexpected_file)
            .unwrap()
            .write_all(b"# Notes")
            .unwrap();

        let result = validate_model_structure(&model_dir);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("Unexpected file"));
    }

    #[test]
    fn test_validate_model_structure_wrong_sql_name() {
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().join("test_model");
        fs::create_dir(&model_dir).unwrap();

        // Create SQL file with wrong name
        let sql_file = model_dir.join("wrong_name.sql");
        let yml_file = model_dir.join("test_model.yml");

        File::create(&sql_file)
            .unwrap()
            .write_all(b"SELECT * FROM table")
            .unwrap();
        File::create(&yml_file)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        let result = validate_model_structure(&model_dir);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2); // Missing correct SQL file + unexpected file
        assert!(result.errors[0].contains("Missing SQL file"));
        assert!(result.errors[1].contains("Unexpected file"));
    }

    #[test]
    fn test_validate_imports_directory_yml_only() {
        let temp_dir = tempdir().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();

        // Create imports directory
        let imports_dir = models_dir.join("imports");
        fs::create_dir(&imports_dir).unwrap();

        // Create raw_data subdirectory
        let raw_data_dir = imports_dir.join("raw_data");
        fs::create_dir(&raw_data_dir).unwrap();

        // Create only YAML file for imports
        let imports_yml = imports_dir.join("imports.yml");
        File::create(&imports_yml)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        // Create only YAML file for raw_data
        let raw_data_yml = raw_data_dir.join("raw_data.yml");
        File::create(&raw_data_yml)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        // Validate imports directory
        let imports_result = validate_model_structure(&imports_dir);

        // Imports directory should be valid with just YAML file
        assert!(
            imports_result.is_valid,
            "Imports directory should be valid with just YAML: {:?}",
            imports_result.errors
        );
        assert!(imports_result.errors.is_empty());

        // Validate raw_data directory
        let raw_data_result = validate_model_structure(&raw_data_dir);

        // raw_data directory should be valid with just YAML file
        assert!(
            raw_data_result.is_valid,
            "Raw data directory should be valid with just YAML: {:?}",
            raw_data_result.errors
        );
        assert!(raw_data_result.errors.is_empty());
    }

    #[test]
    fn test_validate_imports_directory_with_unexpected_file() {
        let temp_dir = tempdir().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();

        // Create imports directory
        let imports_dir = models_dir.join("imports");
        fs::create_dir(&imports_dir).unwrap();

        // Create YAML file for imports
        let imports_yml = imports_dir.join("imports.yml");
        File::create(&imports_yml)
            .unwrap()
            .write_all(b"version: 2")
            .unwrap();

        // Create unexpected file
        let unexpected_file = imports_dir.join("README.md");
        File::create(&unexpected_file)
            .unwrap()
            .write_all(b"# Imports Directory")
            .unwrap();

        // Validate imports directory
        let imports_result = validate_model_structure(&imports_dir);

        // Imports directory should fail validation due to unexpected file
        assert!(!imports_result.is_valid);
        assert_eq!(imports_result.errors.len(), 1);
        assert!(imports_result.errors[0].contains("Unexpected file in imports directory"));
    }
}
