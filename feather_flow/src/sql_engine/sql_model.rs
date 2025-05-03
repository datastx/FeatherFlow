use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlparser::ast::Statement;
use sqlparser::dialect::Dialect;
use sqlparser::parser::Parser as SqlParser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::validators::validate_model_structure;

use super::extractors;
use super::lineage::ColumnLineage;

/// Represents a parsed SQL model file with metadata and dependencies
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SqlModel {
    // Core identification fields
    pub unique_id: String,
    pub name: String,

    // File information
    pub fully_qualified_file_path: PathBuf,
    pub relative_file_path: PathBuf,
    pub file_name: String,
    pub checksum: String,
    pub parent_dir: PathBuf, // Added to track the parent directory

    // SQL content
    pub raw_sql: String,
    pub compiled_sql: Option<String>,

    // AST representation
    pub ast: Vec<Statement>,

    // Dependency information
    pub depends_on: HashSet<String>,
    pub referenced_tables: HashSet<String>,
    pub referenced_sources: HashSet<String>,
    pub upstream_models: HashSet<String>,
    pub downstream_models: HashSet<String>,

    // Metadata
    pub description: Option<String>,
    pub dialect: String,
    pub tags: Vec<String>,
    pub meta: HashMap<String, serde_json::Value>,

    // Model creation specific fields
    pub materialized: Option<String>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub object_name: Option<String>,
    pub alias: Option<String>,

    // Tracking information
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Column information
    pub columns: HashMap<String, ColumnInfo>,

    // Validation information
    pub is_valid_structure: bool,
    pub structure_errors: Vec<String>,
}

/// Information about a column in a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub description: Option<String>,
    pub data_type: Option<String>,
    pub tests: Vec<String>,
    pub meta: HashMap<String, serde_json::Value>,
    pub source_columns: Vec<ColumnLineageInfo>,
}

/// Information about column lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnLineageInfo {
    pub table: String,
    pub column: String,
    pub transformation_type: String,
}

impl SqlModel {
    /// Create a new SqlModel from a file path
    pub fn from_path(
        path: &Path,
        project_root: &Path,
        dialect_name: &str,
        dialect: &dyn Dialect,
    ) -> Result<Self> {
        // Read the file content
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read SQL file: {}", path.display()))?;

        Self::from_content(path, project_root, content, dialect_name, dialect)
    }

    /// Create a new SqlModel from SQL content and path information
    pub fn from_content(
        path: &Path,
        project_root: &Path,
        content: String,
        dialect_name: &str,
        dialect: &dyn Dialect,
    ) -> Result<Self> {
        // Parse SQL to AST
        let ast = SqlParser::parse_sql(dialect, &content)
            .with_context(|| format!("Failed to parse SQL from {}", path.display()))?;

        // Calculate file metadata
        let file_name = path
            .file_name()
            .with_context(|| "File has no name")?
            .to_string_lossy()
            .to_string();

        let name = path
            .file_stem()
            .with_context(|| "File has no stem")?
            .to_string_lossy()
            .to_string();

        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_path_buf();

        // Get parent directory
        let parent_dir = path.parent().unwrap_or(Path::new("")).to_path_buf();

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let checksum = format!("{:x}", hasher.finalize());

        let unique_id = format!(
            "model.{}",
            relative_path
                .to_string_lossy()
                .replace(['/', '\\'], ".")
                .replace(".sql", "")
        );

        let now = Utc::now();

        // Validate the model structure
        let (is_valid_structure, structure_errors) = if parent_dir.exists() {
            let validation_result = validate_model_structure(&parent_dir);
            (validation_result.is_valid, validation_result.errors)
        } else {
            (false, vec!["Parent directory does not exist".to_string()])
        };

        Ok(Self {
            unique_id,
            name,
            fully_qualified_file_path: path.to_path_buf(),
            relative_file_path: relative_path,
            file_name,
            checksum,
            parent_dir,
            raw_sql: content,
            compiled_sql: None,
            ast,
            depends_on: HashSet::new(),
            referenced_tables: HashSet::new(),
            referenced_sources: HashSet::new(),
            upstream_models: HashSet::new(),
            downstream_models: HashSet::new(),
            description: None,
            dialect: dialect_name.to_string(),
            tags: Vec::new(),
            meta: HashMap::new(),
            materialized: None,
            schema: None,
            database: None,
            alias: None,
            object_name: None,
            created_at: now,
            updated_at: now,
            columns: HashMap::new(),
            is_valid_structure,
            structure_errors,
        })
    }

    /// Validates that the model follows the proper file structure pattern
    #[allow(dead_code)]
    pub fn validate_structure(&self) -> Result<()> {
        if !self.is_valid_structure {
            return Err(anyhow!(
                "Invalid model structure for {}: {}",
                self.fully_qualified_file_path.display(),
                self.structure_errors.join(", ")
            ));
        }

        Ok(())
    }

    /// Extract table dependencies from the parsed AST
    pub fn extract_dependencies(&mut self) -> Result<()> {
        // Use the common table extraction utility from extractors module
        self.referenced_tables = extractors::get_external_table_deps_set(&self.ast);
        Ok(())
    }

    /// Extract column-level lineage information
    #[allow(dead_code)]
    pub fn extract_column_lineage(&mut self) -> Result<Vec<ColumnLineage>> {
        // This would leverage your existing column lineage extraction code
        // For now, just a stub
        Ok(Vec::new())
    }

    /// Apply a transformation to the AST and update compiled_sql
    #[allow(dead_code)]
    pub fn modify_ast<F>(&mut self, transformation: F) -> Result<()>
    where
        F: FnOnce(&mut Vec<Statement>),
    {
        // Apply the transformation
        transformation(&mut self.ast);

        // Update compiled_sql
        self.regenerate_sql()
    }

    /// Regenerate SQL from the current AST
    #[allow(dead_code)]
    pub fn regenerate_sql(&mut self) -> Result<()> {
        // Here you would convert the AST back to SQL
        // Similar to your ast_to_sql function in ast_utils.rs
        // For now, just a stub
        self.compiled_sql = Some("-- Regenerated SQL would go here".to_string());
        Ok(())
    }

    /// Generate a new checksum from current content
    #[allow(dead_code)]
    pub fn update_checksum(&mut self) {
        let content = self.compiled_sql.as_ref().unwrap_or(&self.raw_sql);
        let mut hasher = Sha256::new();
        hasher.update(content);
        self.checksum = format!("{:x}", hasher.finalize());
        self.updated_at = Utc::now();
    }
}

/// Collection of all parsed SQL models
#[derive(Debug, Clone, Default)]
pub struct SqlModelCollection {
    models: HashMap<String, SqlModel>,
    child_map: HashMap<String, HashSet<String>>, // parent_id -> child_ids
    parent_map: HashMap<String, HashSet<String>>, // child_id -> parent_ids
}

impl SqlModelCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            child_map: HashMap::new(),
            parent_map: HashMap::new(),
        }
    }

    /// Add a model to the collection
    pub fn add_model(&mut self, model: SqlModel) {
        let id = model.unique_id.clone();
        self.models.insert(id, model);
    }

    /// Get a model by ID
    #[allow(dead_code)]
    pub fn get_model(&self, id: &str) -> Option<&SqlModel> {
        self.models.get(id)
    }

    /// Get a mutable reference to a model
    #[allow(dead_code)]
    pub fn get_model_mut(&mut self, id: &str) -> Option<&mut SqlModel> {
        self.models.get_mut(id)
    }

    /// Build the dependency graph
    pub fn build_dependency_graph(&mut self) {
        self.child_map.clear();
        self.parent_map.clear();

        // First pass: collect all referenced tables
        let model_ids: Vec<String> = self.models.keys().cloned().collect();

        // Map table names to model IDs
        let mut table_to_model: HashMap<String, String> = HashMap::new();
        for id in &model_ids {
            if let Some(model) = self.models.get(id) {
                let table_name = format!(
                    "{}.{}",
                    model.schema.as_deref().unwrap_or("public"),
                    model.name
                );
                table_to_model.insert(table_name, id.clone());
            }
        }

        // Second pass: collect relationships to avoid borrow issues
        let mut relationships: Vec<(String, String)> = Vec::new();
        for id in &model_ids {
            if let Some(model) = self.models.get(id) {
                for ref_table in &model.referenced_tables {
                    if let Some(parent_id) = table_to_model.get(ref_table) {
                        relationships.push((id.clone(), parent_id.clone()));

                        // Add parent-child relationship to maps
                        self.child_map
                            .entry(parent_id.clone())
                            .or_default()
                            .insert(id.clone());

                        self.parent_map
                            .entry(id.clone())
                            .or_default()
                            .insert(parent_id.clone());
                    }
                }
            }
        }

        // Third pass: update model dependency information
        for (child_id, parent_id) in relationships {
            // Update child model's upstream dependencies
            if let Some(child_model) = self.models.get_mut(&child_id) {
                child_model.upstream_models.insert(parent_id.clone());
            }

            // Update parent model's downstream dependencies
            if let Some(parent_model) = self.models.get_mut(&parent_id) {
                parent_model.downstream_models.insert(child_id);
            }
        }
    }

    /// Check for circular dependencies
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        // Implementation would use a depth-first search to find cycles
        // For now, just a stub
        Vec::new()
    }

    /// Get all models in topological order
    pub fn get_execution_order(&self) -> Result<Vec<&SqlModel>> {
        // Implementation would use topological sorting
        // For now, just return all models
        Ok(self.models.values().collect())
    }

    /// Generate a DOT graph representation
    pub fn to_dot_graph(&self) -> String {
        let mut result = String::from("digraph models {\n");
        result.push_str("  rankdir=LR;\n");
        result.push_str("  node [shape=box];\n");

        // Add nodes
        for model in self.models.values() {
            result.push_str(&format!(
                "  \"{}\" [label=\"{}\"];\n",
                model.unique_id, model.name
            ));
        }

        // Add edges
        for (parent_id, children) in &self.child_map {
            for child_id in children {
                result.push_str(&format!("  \"{}\" -> \"{}\";\n", parent_id, child_id));
            }
        }

        result.push_str("}\n");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::DuckDbDialect;
    use std::path::PathBuf;

    #[test]
    fn test_create_model_from_content() {
        let sql = "SELECT id, name FROM users";
        let path = PathBuf::from("/tmp/test_model.sql");
        let project_root = PathBuf::from("/tmp");
        let dialect = DuckDbDialect {};

        let model =
            SqlModel::from_content(&path, &project_root, sql.to_string(), "duckdb", &dialect)
                .unwrap();

        assert_eq!(model.name, "test_model");
        assert_eq!(model.file_name, "test_model.sql");
        assert_eq!(model.raw_sql, sql);
        assert!(!model.checksum.is_empty());
        assert_eq!(model.ast.len(), 1); // Should have one statement
    }

    #[test]
    fn test_extract_dependencies() {
        let sql = "SELECT id, name FROM schema1.users JOIN schema2.orders ON schema1.users.id = schema2.orders.user_id";
        let path = PathBuf::from("/tmp/test_model.sql");
        let project_root = PathBuf::from("/tmp");
        let dialect = DuckDbDialect {};

        let mut model =
            SqlModel::from_content(&path, &project_root, sql.to_string(), "duckdb", &dialect)
                .unwrap();

        model.extract_dependencies().unwrap();

        assert!(model.referenced_tables.contains("schema1.users"));
        assert!(model.referenced_tables.contains("schema2.orders"));
        assert_eq!(model.referenced_tables.len(), 2);
    }
}
