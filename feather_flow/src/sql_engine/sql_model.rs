use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_yaml;
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

/// YAML model configuration structure (top level)
#[derive(Debug, Serialize, Deserialize)]
struct YamlConfig {
    version: i32,
    models: Option<Vec<YamlModel>>,
    sources: Option<Vec<YamlSource>>,
}

/// YAML model definition
#[derive(Debug, Serialize, Deserialize)]
struct YamlModel {
    name: String,
    description: Option<String>,
    meta: Option<HashMap<String, serde_json::Value>>,
    config: Option<YamlModelConfig>,
    database_name: Option<String>,
    schema_name: Option<String>,
    object_name: Option<String>,
    columns: Option<Vec<YamlColumn>>,
}

/// YAML source definition for external data sources
#[derive(Debug, Serialize, Deserialize)]
struct YamlSource {
    name: String,
    description: Option<String>,
    database: String,
    tables: Vec<YamlSourceTable>,
}

/// YAML source table definition
#[derive(Debug, Serialize, Deserialize)]
struct YamlSourceTable {
    name: String,
    description: Option<String>,
    columns: Option<Vec<YamlColumn>>,
}

/// YAML model configuration options
#[derive(Debug, Serialize, Deserialize)]
struct YamlModelConfig {
    materialized: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

/// YAML column definition
#[derive(Debug, Serialize, Deserialize)]
struct YamlColumn {
    name: String,
    description: Option<String>,
    data_type: Option<String>,
    tests: Option<Vec<String>>,
    meta: Option<HashMap<String, serde_json::Value>>,
}

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
    pub external_sources: HashSet<String>, // Cache for external sources
    pub depth: Option<usize>,              // Graph depth for execution scheduling

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

    /// Get external sources that are not project models
    pub fn get_external_sources(&self) -> HashSet<String> {
        // Return the cached external sources
        self.external_sources.clone()
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

        let mut model = Self {
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
            external_sources: HashSet::new(),
            depth: None,
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
        };

        // Load YAML metadata if available
        if model.is_valid_structure {
            // Ignore errors when loading YAML - this allows parsing to continue even if YAML is invalid
            let _ = model.load_yaml_metadata();
        }

        Ok(model)
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

    /// Load metadata from the corresponding YAML file
    pub fn load_yaml_metadata(&mut self) -> Result<()> {
        // Construct path to the YAML file
        let yaml_path = self.parent_dir.join(format!("{}.yml", self.name));

        // Check if the YAML file exists
        if !yaml_path.exists() {
            return Ok(()); // Not an error, just no metadata to load
        }

        // Read and parse YAML file
        let yaml_content = fs::read_to_string(&yaml_path)
            .with_context(|| format!("Failed to read YAML file: {}", yaml_path.display()))?;

        let yaml_config: YamlConfig = serde_yaml::from_str(&yaml_content)
            .with_context(|| format!("Failed to parse YAML from {}", yaml_path.display()))?;

        // Find the model configuration that matches this model's name
        if let Some(models) = &yaml_config.models {
            for model_config in models {
                if model_config.name == self.name {
                    // Update model with YAML configuration
                    self.description = model_config.description.clone();

                    if let Some(meta) = &model_config.meta {
                        self.meta = meta.clone();

                        // Extract tags from meta if available
                        if let Some(tags) = self.meta.get("tags") {
                            if let Some(tags_array) = tags.as_array() {
                                self.tags = tags_array
                                    .iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect();
                            }
                        }
                    }

                    // Set materialization configuration
                    if let Some(config) = &model_config.config {
                        self.materialized = config.materialized.clone();
                    }

                    // Set database/schema/object information
                    self.database = model_config.database_name.clone();
                    self.schema = model_config.schema_name.clone();
                    self.object_name = model_config.object_name.clone();

                    // Load column information
                    if let Some(yaml_columns) = &model_config.columns {
                        for yaml_col in yaml_columns {
                            let column_info = ColumnInfo {
                                name: yaml_col.name.clone(),
                                description: yaml_col.description.clone(),
                                data_type: yaml_col.data_type.clone(),
                                tests: yaml_col.tests.clone().unwrap_or_default(),
                                meta: yaml_col.meta.clone().unwrap_or_default(),
                                source_columns: Vec::new(), // Will be populated by column lineage
                            };

                            self.columns.insert(column_info.name.clone(), column_info);
                        }
                    }

                    break; // Found our model, no need to continue
                }
            }
        }

        Ok(())
    }
}

/// Collection of all parsed SQL models
#[derive(Debug, Clone, Default)]
pub struct SqlModelCollection {
    models: HashMap<String, SqlModel>,
    child_map: HashMap<String, HashSet<String>>, // parent_id -> child_ids
    parent_map: HashMap<String, HashSet<String>>, // child_id -> parent_ids
    defined_imports: HashSet<String>, // Set of external imports defined in imports directory
    missing_imports: HashMap<String, HashSet<String>>, // model_id -> missing imports
}

impl SqlModelCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            child_map: HashMap::new(),
            parent_map: HashMap::new(),
            defined_imports: HashSet::new(),
            missing_imports: HashMap::new(),
        }
    }

    /// Add a model to the collection
    pub fn add_model(&mut self, model: SqlModel) {
        let id = model.unique_id.clone();
        self.models.insert(id, model);
    }

    /// Load import definitions from the imports directory
    pub fn load_source_definitions(&mut self, project_root: &Path) -> std::io::Result<()> {
        // Check if project_root already contains "models" in its path
        let mut imports_dir = project_root.to_path_buf();
        if !project_root.ends_with("models") {
            imports_dir = imports_dir.join("models");
        }
        imports_dir = imports_dir.join("imports");

        if !imports_dir.exists() {
            eprintln!(
                "Warning: Imports directory not found at: {}",
                imports_dir.display()
            );
            return Ok(()); // No imports directory, nothing to load
        }

        // Clear existing definitions
        self.defined_imports.clear();

        // Walk the imports directory to find all YAML files
        use walkdir::WalkDir;

        eprintln!("Scanning imports directory: {}", imports_dir.display());
        let mut yaml_files_found = 0;

        for entry in WalkDir::new(&imports_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "yml"))
        {
            let yaml_path = entry.path();
            yaml_files_found += 1;
            eprintln!("Found YAML file: {}", yaml_path.display());

            let yaml_content = fs::read_to_string(yaml_path)?;

            // Parse YAML file
            if let Ok(yaml_config) = serde_yaml::from_str::<YamlConfig>(&yaml_content) {
                // Process source definitions if present
                if let Some(sources) = yaml_config.sources {
                    eprintln!("Found {} sources in {}", sources.len(), yaml_path.display());

                    for source in sources {
                        let source_prefix = source.database.to_string();
                        eprintln!(
                            "Processing import: {} (database: {})",
                            source.name, source_prefix
                        );

                        // Add each table from this import to the defined imports set
                        for table in &source.tables {
                            let import_name = format!("{}.{}", source_prefix, table.name);
                            eprintln!("  Adding import: {}", import_name);
                            self.defined_imports.insert(import_name);
                        }
                    }
                } else {
                    eprintln!("No imports found in {}", yaml_path.display());
                }
            } else {
                eprintln!("Failed to parse YAML file: {}", yaml_path.display());
            }
        }

        eprintln!("Found {} YAML files in imports directory", yaml_files_found);
        eprintln!("Loaded {} defined imports", self.defined_imports.len());

        // Print all defined imports for debugging
        if !self.defined_imports.is_empty() {
            eprintln!("Defined imports:");
            for import in &self.defined_imports {
                eprintln!("  {}", import);
            }
        }

        Ok(())
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
        self.missing_imports.clear();

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

        // Fourth pass: calculate external sources for each model and validate them
        for id in &model_ids {
            if let Some(model) = self.models.get_mut(id) {
                // Identify external sources (tables that don't map to known models)
                let mut external_sources = HashSet::new();
                let mut missing_external_sources = HashSet::new();

                for ref_table in &model.referenced_tables {
                    if !table_to_model.contains_key(ref_table) {
                        external_sources.insert(ref_table.clone());

                        // Check if this external source is defined in imports
                        if !self.defined_imports.contains(ref_table) {
                            missing_external_sources.insert(ref_table.clone());
                        }
                    }
                }

                // Store missing imports for this model if any
                if !missing_external_sources.is_empty() {
                    self.missing_imports
                        .insert(id.clone(), missing_external_sources);
                }

                model.external_sources = external_sources;
            }
        }

        // Calculate model depths for execution scheduling
        self.calculate_model_depths();
    }

    /// Check for circular dependencies
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        // Implementation would use a depth-first search to find cycles
        // For now, just a stub
        Vec::new()
    }

    /// Check if any models reference undefined external imports
    pub fn has_missing_sources(&self) -> bool {
        !self.missing_imports.is_empty()
    }

    /// Get a map of model IDs to their missing external imports
    #[allow(dead_code)]
    pub fn get_missing_sources(&self) -> &HashMap<String, HashSet<String>> {
        &self.missing_imports
    }

    /// Get a formatted report of missing external imports
    pub fn get_missing_sources_report(&self) -> Vec<String> {
        let mut report = Vec::new();

        for (model_id, missing_sources) in &self.missing_imports {
            if let Some(model) = self.models.get(model_id) {
                let missing_list = missing_sources
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<_>>()
                    .join(", ");

                report.push(format!(
                    "Model '{}' references undefined external import(s): {}",
                    model.name, missing_list
                ));
            }
        }

        report
    }

    /// Get all models in topological order
    pub fn get_execution_order(&self) -> Result<Vec<&SqlModel>> {
        // Implementation would use topological sorting
        // For now, just return all models
        Ok(self.models.values().collect())
    }

    /// Calculate the depth of each model in the dependency graph
    /// Depth is defined as:
    /// - 0: Nodes with no upstream dependencies (source nodes)
    /// - 1+: Maximum depth of upstream dependencies + 1
    pub fn calculate_model_depths(&mut self) {
        // Reset all depths first
        for model in self.models.values_mut() {
            model.depth = None;
        }

        // Get all model IDs
        let model_ids: Vec<String> = self.models.keys().cloned().collect();

        // First pass: mark source nodes (no upstream dependencies) as depth 0
        for id in &model_ids {
            if let Some(model) = self.models.get_mut(id) {
                if model.upstream_models.is_empty() {
                    model.depth = Some(0);
                }
            }
        }

        // Iteratively calculate depths until all models have depths assigned
        let mut made_changes = true;
        while made_changes {
            made_changes = false;

            for id in &model_ids {
                let mut model_needs_update = false;
                let mut max_upstream_depth: Option<usize> = None;

                // Skip if model already has a depth assigned
                if let Some(model) = self.models.get(id) {
                    if model.depth.is_some() {
                        continue;
                    }

                    // Check if all upstream models have depths
                    let mut all_upstreams_have_depths = true;
                    for upstream_id in &model.upstream_models {
                        if let Some(upstream) = self.models.get(upstream_id) {
                            if let Some(depth) = upstream.depth {
                                max_upstream_depth =
                                    Some(max_upstream_depth.unwrap_or(0).max(depth));
                            } else {
                                all_upstreams_have_depths = false;
                                break;
                            }
                        }
                    }

                    // If all upstream models have depths, calculate this model's depth
                    if all_upstreams_have_depths && !model.upstream_models.is_empty() {
                        model_needs_update = true;
                    }
                }

                // Update model depth (done separately to avoid borrow issues)
                if model_needs_update {
                    if let Some(model) = self.models.get_mut(id) {
                        model.depth = max_upstream_depth.map(|d| d + 1);
                        made_changes = true;
                    }
                }
            }
        }
    }

    /// Generate a DOT graph representation
    pub fn to_dot_graph(&self) -> String {
        let mut result = String::from("digraph models {\n");
        result.push_str("  rankdir=LR;\n");
        result.push_str("  node [shape=box];\n");

        // Add nodes with depth information
        for model in self.models.values() {
            let depth_label = model.depth.map_or("?".to_string(), |d| d.to_string());
            result.push_str(&format!(
                "  \"{}\" [label=\"{} (depth: {})\"];\n",
                model.unique_id, model.name, depth_label
            ));
        }

        // Add edges
        for (parent_id, children) in &self.child_map {
            for child_id in children {
                result.push_str(&format!("  \"{}\" -> \"{}\";\n", parent_id, child_id));
            }
        }

        // Add subgraphs for depth levels
        let max_depth = self
            .models
            .values()
            .filter_map(|m| m.depth)
            .max()
            .unwrap_or(0);

        for depth in 0..=max_depth {
            result.push_str(&format!("  subgraph depth_{} {{\n", depth));
            result.push_str("    rank=same;\n");

            // Add nodes at this depth level
            for model in self.models.values() {
                if model.depth == Some(depth) {
                    result.push_str(&format!("    \"{}\";\n", model.unique_id));
                }
            }

            result.push_str("  }\n");
        }

        result.push_str("}\n");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::DuckDbDialect;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_model_depth_calculation() {
        let temp_dir = tempdir().unwrap();
        let project_root = temp_dir.path();
        let dialect = DuckDbDialect {};

        // Create SQL files for a simple dependency chain A <- B <- C
        // A is source, C is terminal

        // Source model A (depth 0)
        let model_a_dir = project_root.join("model_a");
        fs::create_dir(&model_a_dir).unwrap();
        let sql_a = "SELECT id, name FROM external_source";
        let file_a = model_a_dir.join("model_a.sql");
        fs::write(&file_a, sql_a).unwrap();

        // Intermediate model B (depth 1)
        let model_b_dir = project_root.join("model_b");
        fs::create_dir(&model_b_dir).unwrap();
        let sql_b = "SELECT id, name FROM public.model_a WHERE active = true";
        let file_b = model_b_dir.join("model_b.sql");
        fs::write(&file_b, sql_b).unwrap();

        // Terminal model C (depth 2)
        let model_c_dir = project_root.join("model_c");
        fs::create_dir(&model_c_dir).unwrap();
        let sql_c =
            "SELECT a.id, b.name FROM public.model_a a JOIN public.model_b b ON a.id = b.id";
        let file_c = model_c_dir.join("model_c.sql");
        fs::write(&file_c, sql_c).unwrap();

        // Create model collection and parse models
        let mut model_collection = SqlModelCollection::new();

        let mut model_a = SqlModel::from_path(&file_a, project_root, "duckdb", &dialect).unwrap();
        model_a.extract_dependencies().unwrap();
        model_collection.add_model(model_a);

        let mut model_b = SqlModel::from_path(&file_b, project_root, "duckdb", &dialect).unwrap();
        model_b.extract_dependencies().unwrap();
        model_collection.add_model(model_b);

        let mut model_c = SqlModel::from_path(&file_c, project_root, "duckdb", &dialect).unwrap();
        model_c.extract_dependencies().unwrap();
        model_collection.add_model(model_c);

        // Build dependency graph (this will calculate depths)
        model_collection.build_dependency_graph();

        // Get models by ID
        let model_a_id = format!("model.model_a.model_a");
        let model_b_id = format!("model.model_b.model_b");
        let model_c_id = format!("model.model_c.model_c");

        // Verify depths
        if let Some(model_a) = model_collection.get_model(&model_a_id) {
            assert_eq!(model_a.depth, Some(0), "Source model A should have depth 0");
        } else {
            panic!("Model A not found");
        }

        if let Some(model_b) = model_collection.get_model(&model_b_id) {
            assert_eq!(
                model_b.depth,
                Some(1),
                "Intermediate model B should have depth 1"
            );
        } else {
            panic!("Model B not found");
        }

        if let Some(model_c) = model_collection.get_model(&model_c_id) {
            assert_eq!(
                model_c.depth,
                Some(2),
                "Terminal model C should have depth 2 as it depends on model B which has depth 1"
            );
        } else {
            panic!("Model C not found");
        }
    }

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

    #[test]
    fn test_load_yaml_metadata() {
        // Create a temporary directory with SQL and YAML files
        let temp_dir = tempdir().unwrap();
        let model_dir = temp_dir.path().join("test_model");
        fs::create_dir(&model_dir).unwrap();

        // Create SQL file
        let sql_file = model_dir.join("test_model.sql");
        fs::write(&sql_file, "SELECT id, name FROM users").unwrap();

        // Create YAML file with metadata
        let yaml_content = r#"
version: 2

models:
  - name: test_model
    description: A test model for unit testing
    meta:
      owner: "test_team"
      tags: ["test", "example"]
    config:
      materialized: table
    database_name: test_db
    schema_name: test_schema
    object_name: test_model_table
    columns:
      - name: id
        description: The primary key
        data_type: integer
      - name: name
        description: The user's name
        data_type: string
"#;
        let yaml_file = model_dir.join("test_model.yml");
        fs::write(&yaml_file, yaml_content).unwrap();

        // Create and parse the model
        let dialect = DuckDbDialect {};
        let model = SqlModel::from_path(&sql_file, temp_dir.path(), "duckdb", &dialect).unwrap();

        // Test that validation passes
        assert!(model.is_valid_structure);

        // Test that YAML metadata was loaded correctly
        assert_eq!(
            model.description,
            Some("A test model for unit testing".to_string())
        );
        assert_eq!(model.materialized, Some("table".to_string()));
        assert_eq!(model.database, Some("test_db".to_string()));
        assert_eq!(model.schema, Some("test_schema".to_string()));
        assert_eq!(model.object_name, Some("test_model_table".to_string()));

        // Check tags
        assert_eq!(model.tags, vec!["test".to_string(), "example".to_string()]);

        // Check columns
        assert_eq!(model.columns.len(), 2);
        assert!(model.columns.contains_key("id"));
        assert!(model.columns.contains_key("name"));

        let id_column = model.columns.get("id").unwrap();
        assert_eq!(id_column.description, Some("The primary key".to_string()));
        assert_eq!(id_column.data_type, Some("integer".to_string()));

        let name_column = model.columns.get("name").unwrap();
        assert_eq!(name_column.description, Some("The user's name".to_string()));
        assert_eq!(name_column.data_type, Some("string".to_string()));
    }
}
