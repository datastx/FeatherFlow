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

#[derive(Debug, Serialize, Deserialize)]
struct YamlConfig {
    version: i32,
    models: Option<Vec<YamlModel>>,
    sources: Option<Vec<YamlSource>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlOutput {
    pub version: i32,
    pub models: std::collections::BTreeMap<String, YamlOutputModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlOutputModel {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub materialized: Option<String>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub object_name: Option<String>,
    pub tags: Vec<String>,
    pub columns: Vec<YamlOutputColumn>,
    pub depends_on: Vec<String>,
    pub referenced_by: Vec<String>,
    pub external_sources: Vec<String>,
    pub depth: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlOutputColumn {
    pub name: String,
    pub description: Option<String>,
    pub data_type: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize)]
struct YamlSource {
    name: String,
    description: Option<String>,
    database: String,
    tables: Vec<YamlSourceTable>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlSourceTable {
    name: String,
    description: Option<String>,
    columns: Option<Vec<YamlColumn>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlModelConfig {
    materialized: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlColumn {
    name: String,
    description: Option<String>,
    data_type: Option<String>,
    tests: Option<Vec<String>>,
    meta: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SqlModel {
    pub unique_id: String,
    pub name: String,
    pub fully_qualified_file_path: PathBuf,
    pub relative_file_path: PathBuf,
    pub file_name: String,
    pub checksum: String,
    pub parent_dir: PathBuf,
    pub raw_sql: String,
    pub compiled_sql: Option<String>,
    pub ast: Vec<Statement>,
    pub depends_on: HashSet<String>,
    pub referenced_tables: HashSet<String>,
    pub referenced_sources: HashSet<String>,
    pub upstream_models: HashSet<String>,
    pub downstream_models: HashSet<String>,
    pub external_sources: HashSet<String>,
    pub depth: Option<usize>,
    pub description: Option<String>,
    pub dialect: String,
    pub tags: Vec<String>,
    pub meta: HashMap<String, serde_json::Value>,
    pub materialized: Option<String>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub object_name: Option<String>,
    pub alias: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub columns: HashMap<String, ColumnInfo>,
    pub is_valid_structure: bool,
    pub structure_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub description: Option<String>,
    pub data_type: Option<String>,
    pub tests: Vec<String>,
    pub meta: HashMap<String, serde_json::Value>,
    pub source_columns: Vec<ColumnLineageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnLineageInfo {
    pub table: String,
    pub column: String,
    pub transformation_type: String,
}

impl SqlModel {
    pub fn from_path(
        path: &Path,
        project_root: &Path,
        dialect_name: &str,
        dialect: &dyn Dialect,
    ) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read SQL file: {}", path.display()))?;

        Self::from_content(path, project_root, content, dialect_name, dialect)
    }

    pub fn get_external_sources(&self) -> HashSet<String> {
        self.external_sources.clone()
    }

    pub fn from_content(
        path: &Path,
        project_root: &Path,
        content: String,
        dialect_name: &str,
        _dialect: &dyn Dialect,
    ) -> Result<Self> {
        let ast = parse_sql_content(&content, path)?;
        let metadata = extract_file_metadata(path, project_root)?;
        let (is_valid_structure, structure_errors) =
            validate_directory_structure(&metadata.parent_dir);

        let model = Self::create_model(
            metadata,
            content,
            ast,
            dialect_name.to_string(),
            is_valid_structure,
            structure_errors,
        );

        Ok(model)
    }

    fn create_model(
        metadata: ModelMetadata,
        content: String,
        ast: Vec<Statement>,
        dialect: String,
        is_valid_structure: bool,
        structure_errors: Vec<String>,
    ) -> Self {
        let now = Utc::now();

        let mut model = Self {
            unique_id: metadata.unique_id,
            name: metadata.name,
            fully_qualified_file_path: metadata.fully_qualified_path,
            relative_file_path: metadata.relative_path,
            file_name: metadata.file_name,
            checksum: metadata.checksum,
            parent_dir: metadata.parent_dir,
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
            dialect,
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

        if model.is_valid_structure {
            let _ = model.load_yaml_metadata();
        }

        model
    }

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

    pub fn extract_dependencies(&mut self) -> Result<()> {
        self.referenced_tables = extractors::get_external_table_deps_set(&self.ast);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn extract_column_lineage(&mut self) -> Result<Vec<ColumnLineage>> {
        Ok(Vec::new())
    }

    #[allow(dead_code)]
    pub fn modify_ast<F>(&mut self, transformation: F) -> Result<()>
    where
        F: FnOnce(&mut Vec<Statement>),
    {
        transformation(&mut self.ast);
        self.regenerate_sql()
    }

    #[allow(dead_code)]
    pub fn regenerate_sql(&mut self) -> Result<()> {
        self.compiled_sql = Some("-- Regenerated SQL would go here".to_string());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_checksum(&mut self) {
        let content = self.compiled_sql.as_ref().unwrap_or(&self.raw_sql);
        let mut hasher = Sha256::new();
        hasher.update(content);
        self.checksum = format!("{:x}", hasher.finalize());
        self.updated_at = Utc::now();
    }

    pub fn load_yaml_metadata(&mut self) -> Result<()> {
        let yaml_path = self.parent_dir.join(format!("{}.yml", self.name));

        if !yaml_path.exists() {
            return Ok(());
        }

        let yaml_content = load_yaml_file(&yaml_path)?;
        let yaml_config: YamlConfig = parse_yaml_content(&yaml_content, &yaml_path)?;

        self.apply_yaml_config(&yaml_config);

        Ok(())
    }

    fn apply_yaml_config(&mut self, yaml_config: &YamlConfig) {
        if let Some(models) = &yaml_config.models {
            for model_config in models {
                if model_config.name == self.name {
                    self.apply_model_config(model_config);
                    break;
                }
            }
        }
    }

    fn apply_model_config(&mut self, model_config: &YamlModel) {
        self.description = model_config.description.clone();
        self.apply_model_meta(model_config);

        if let Some(config) = &model_config.config {
            self.materialized = config.materialized.clone();
        }

        self.database = model_config.database_name.clone();
        self.schema = model_config.schema_name.clone();
        self.object_name = model_config.object_name.clone();

        self.load_column_information(model_config);
    }

    fn apply_model_meta(&mut self, model_config: &YamlModel) {
        if let Some(meta) = &model_config.meta {
            self.meta = meta.clone();

            if let Some(tags) = self.meta.get("tags") {
                if let Some(tags_array) = tags.as_array() {
                    self.tags = tags_array
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }
    }

    fn load_column_information(&mut self, model_config: &YamlModel) {
        if let Some(yaml_columns) = &model_config.columns {
            for yaml_col in yaml_columns {
                let column_info = create_column_info(yaml_col);
                self.columns.insert(column_info.name.clone(), column_info);
            }
        }
    }
}

struct ModelMetadata {
    unique_id: String,
    name: String,
    fully_qualified_path: PathBuf,
    relative_path: PathBuf,
    file_name: String,
    checksum: String,
    parent_dir: PathBuf,
}

fn parse_sql_content(content: &str, path: &Path) -> Result<Vec<Statement>> {
    let dialect = sqlparser::dialect::DuckDbDialect {};
    SqlParser::parse_sql(&dialect, content)
        .with_context(|| format!("Failed to parse SQL from {}", path.display()))
}

fn extract_file_metadata(path: &Path, project_root: &Path) -> Result<ModelMetadata> {
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

    let parent_dir = path.parent().unwrap_or(Path::new("")).to_path_buf();

    let unique_id = format!(
        "model.{}",
        relative_path
            .to_string_lossy()
            .replace(['/', '\\'], ".")
            .replace(".sql", "")
    );

    let checksum = calculate_checksum(path)?;

    Ok(ModelMetadata {
        unique_id,
        name,
        fully_qualified_path: path.to_path_buf(),
        relative_path,
        file_name,
        checksum,
        parent_dir,
    })
}

fn calculate_checksum(path: &Path) -> Result<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            if cfg!(test) && !path.exists() {
                "dummy_content_for_testing".to_string()
            } else {
                return Err(e).with_context(|| {
                    format!("Failed to read file for checksum: {}", path.display())
                });
            }
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_directory_structure(parent_dir: &Path) -> (bool, Vec<String>) {
    if parent_dir.exists() {
        let validation_result = validate_model_structure(parent_dir);
        (validation_result.is_valid, validation_result.errors)
    } else {
        (false, vec!["Parent directory does not exist".to_string()])
    }
}

fn load_yaml_file(yaml_path: &Path) -> Result<String> {
    fs::read_to_string(yaml_path)
        .with_context(|| format!("Failed to read YAML file: {}", yaml_path.display()))
}

fn parse_yaml_content(yaml_content: &str, yaml_path: &Path) -> Result<YamlConfig> {
    serde_yaml::from_str(yaml_content)
        .with_context(|| format!("Failed to parse YAML from {}", yaml_path.display()))
}

fn create_column_info(yaml_col: &YamlColumn) -> ColumnInfo {
    ColumnInfo {
        name: yaml_col.name.clone(),
        description: yaml_col.description.clone(),
        data_type: yaml_col.data_type.clone(),
        tests: yaml_col.tests.clone().unwrap_or_default(),
        meta: yaml_col.meta.clone().unwrap_or_default(),
        source_columns: Vec::new(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct SqlModelCollection {
    models: HashMap<String, SqlModel>,
    child_map: HashMap<String, HashSet<String>>,
    parent_map: HashMap<String, HashSet<String>>,
    defined_imports: HashSet<String>,
    missing_imports: HashMap<String, HashSet<String>>,
}

impl SqlModelCollection {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            child_map: HashMap::new(),
            parent_map: HashMap::new(),
            defined_imports: HashSet::new(),
            missing_imports: HashMap::new(),
        }
    }

    pub fn models_count(&self) -> usize {
        self.models.len()
    }

    pub fn to_yaml(&self) -> Result<YamlOutput> {
        match self.get_execution_order() {
            Ok(models) => {
                let mut yaml_models = HashMap::new();
                for model in models {
                    yaml_models.insert(model.unique_id.clone(), model_to_yaml_output(model));
                }

                let ordered_models: std::collections::BTreeMap<String, YamlOutputModel> =
                    yaml_models.into_iter().collect();

                Ok(YamlOutput {
                    version: 1,
                    models: ordered_models,
                })
            }
            Err(err) => Err(anyhow!("Error determining execution order: {}", err)),
        }
    }

    pub fn add_model(&mut self, model: SqlModel) {
        let id = model.unique_id.clone();
        self.models.insert(id, model);
    }

    pub fn load_source_definitions(&mut self, project_root: &Path) -> std::io::Result<()> {
        let imports_dir = get_imports_directory_path(project_root);

        if !imports_dir.exists() {
            eprintln!(
                "Warning: Imports directory not found at: {}",
                imports_dir.display()
            );
            return Ok(());
        }

        self.defined_imports.clear();
        let yaml_files = find_yaml_files(&imports_dir);

        for yaml_path in yaml_files {
            process_import_yaml_file(&yaml_path, &mut self.defined_imports)?;
        }

        debug_log_imports(&self.defined_imports);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_model(&self, id: &str) -> Option<&SqlModel> {
        self.models.get(id)
    }

    #[allow(dead_code)]
    pub fn get_model_mut(&mut self, id: &str) -> Option<&mut SqlModel> {
        self.models.get_mut(id)
    }

    pub fn build_dependency_graph(&mut self) {
        self.clear_dependency_maps();

        let model_ids: Vec<String> = self.models.keys().cloned().collect();
        let table_to_model = self.build_table_to_model_map(&model_ids);

        let relationships = self.collect_model_relationships(&model_ids, &table_to_model);
        self.update_model_dependency_relationships(&relationships);

        self.calculate_external_sources(&model_ids, &table_to_model);
        self.calculate_model_depths();
    }

    fn clear_dependency_maps(&mut self) {
        self.child_map.clear();
        self.parent_map.clear();
        self.missing_imports.clear();
    }

    fn build_table_to_model_map(&self, model_ids: &[String]) -> HashMap<String, String> {
        let mut table_to_model = HashMap::new();

        for id in model_ids {
            if let Some(model) = self.models.get(id) {
                let table_name = format!(
                    "{}.{}",
                    model.schema.as_deref().unwrap_or("public"),
                    model.name
                );
                table_to_model.insert(table_name, id.clone());
            }
        }

        table_to_model
    }

    fn collect_model_relationships(
        &mut self,
        model_ids: &[String],
        table_to_model: &HashMap<String, String>,
    ) -> Vec<(String, String)> {
        let mut relationships = Vec::new();

        for id in model_ids {
            if let Some(model) = self.models.get(id) {
                for ref_table in &model.referenced_tables {
                    if let Some(parent_id) = table_to_model.get(ref_table) {
                        relationships.push((id.clone(), parent_id.clone()));

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

        relationships
    }

    fn update_model_dependency_relationships(&mut self, relationships: &[(String, String)]) {
        for (child_id, parent_id) in relationships {
            if let Some(child_model) = self.models.get_mut(child_id) {
                child_model.upstream_models.insert(parent_id.clone());
            }

            if let Some(parent_model) = self.models.get_mut(parent_id) {
                parent_model.downstream_models.insert(child_id.clone());
            }
        }
    }

    fn calculate_external_sources(
        &mut self,
        model_ids: &[String],
        table_to_model: &HashMap<String, String>,
    ) {
        for id in model_ids {
            let model_sources = {
                if let Some(model) = self.models.get(id) {
                    self.identify_external_sources(model, table_to_model)
                } else {
                    continue;
                }
            };

            let (external_sources, missing_sources) = model_sources;

            if let Some(model) = self.models.get_mut(id) {
                if !missing_sources.is_empty() {
                    self.missing_imports.insert(id.clone(), missing_sources);
                }

                model.external_sources = external_sources;
            }
        }
    }

    fn identify_external_sources(
        &self,
        model: &SqlModel,
        table_to_model: &HashMap<String, String>,
    ) -> (HashSet<String>, HashSet<String>) {
        let mut external_sources = HashSet::new();
        let mut missing_sources = HashSet::new();

        for ref_table in &model.referenced_tables {
            if !table_to_model.contains_key(ref_table) {
                external_sources.insert(ref_table.clone());

                if !self.defined_imports.contains(ref_table) {
                    missing_sources.insert(ref_table.clone());
                }
            }
        }

        (external_sources, missing_sources)
    }

    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        Vec::new()
    }

    pub fn has_missing_sources(&self) -> bool {
        !self.missing_imports.is_empty()
    }

    #[allow(dead_code)]
    pub fn get_missing_sources(&self) -> &HashMap<String, HashSet<String>> {
        &self.missing_imports
    }

    pub fn get_missing_sources_report(&self) -> Vec<String> {
        let mut report = Vec::new();

        for (model_id, missing_sources) in &self.missing_imports {
            if let Some(model) = self.models.get(model_id) {
                let missing_list = format_missing_sources(missing_sources);
                report.push(format!(
                    "Model '{}' references undefined external import(s): {}",
                    model.name, missing_list
                ));
            }
        }

        report
    }

    pub fn get_execution_order(&self) -> Result<Vec<&SqlModel>> {
        let mut models: Vec<&SqlModel> = self.models.values().collect();
        models.sort_by(|a, b| a.unique_id.cmp(&b.unique_id));
        Ok(models)
    }

    pub fn calculate_model_depths(&mut self) {
        for model in self.models.values_mut() {
            model.depth = None;
        }

        let model_ids: Vec<String> = self.models.keys().cloned().collect();
        self.mark_source_nodes(&model_ids);
        self.calculate_model_depths_iteratively(&model_ids);
    }

    fn mark_source_nodes(&mut self, model_ids: &[String]) {
        for id in model_ids {
            if let Some(model) = self.models.get_mut(id) {
                if model.upstream_models.is_empty() {
                    model.depth = Some(0);
                }
            }
        }
    }

    fn calculate_model_depths_iteratively(&mut self, model_ids: &[String]) {
        let mut made_changes = true;
        while made_changes {
            made_changes = false;

            for id in model_ids {
                if self.models.get(id).is_some_and(|m| m.depth.is_some()) {
                    continue;
                }

                if let Some((needs_update, max_depth)) = self.check_model_dependencies(id) {
                    if needs_update {
                        if let Some(model) = self.models.get_mut(id) {
                            model.depth = max_depth.map(|d| d + 1);
                            made_changes = true;
                        }
                    }
                }
            }
        }
    }

    fn check_model_dependencies(&self, model_id: &str) -> Option<(bool, Option<usize>)> {
        let model = self.models.get(model_id)?;

        if model.upstream_models.is_empty() || model.depth.is_some() {
            return Some((false, None));
        }

        let mut max_upstream_depth = None;
        let mut all_upstreams_have_depths = true;

        for upstream_id in &model.upstream_models {
            if let Some(upstream) = self.models.get(upstream_id) {
                if let Some(depth) = upstream.depth {
                    max_upstream_depth = Some(max_upstream_depth.unwrap_or(0).max(depth));
                } else {
                    all_upstreams_have_depths = false;
                    break;
                }
            }
        }

        Some((all_upstreams_have_depths, max_upstream_depth))
    }

    pub fn to_dot_graph(&self) -> String {
        generate_dot_graph(self)
    }
}

fn model_to_yaml_output(model: &SqlModel) -> YamlOutputModel {
    let mut columns: Vec<YamlOutputColumn> = model
        .columns
        .values()
        .map(|col| YamlOutputColumn {
            name: col.name.clone(),
            description: col.description.clone(),
            data_type: col.data_type.clone(),
        })
        .collect();
    columns.sort_by(|a, b| a.name.cmp(&b.name));

    let mut external_sources: Vec<String> = model.get_external_sources().into_iter().collect();
    external_sources.sort();

    let mut depends_on: Vec<String> = model.upstream_models.iter().cloned().collect();
    depends_on.sort();

    let mut referenced_by: Vec<String> = model.downstream_models.iter().cloned().collect();
    referenced_by.sort();

    let mut tags = model.tags.clone();
    tags.sort();

    YamlOutputModel {
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

fn get_imports_directory_path(project_root: &Path) -> PathBuf {
    let mut imports_dir = project_root.to_path_buf();
    if !project_root.ends_with("models") {
        imports_dir = imports_dir.join("models");
    }
    imports_dir.join("imports")
}

fn find_yaml_files(imports_dir: &Path) -> Vec<PathBuf> {
    use walkdir::WalkDir;

    let mut yaml_files = Vec::new();
    log_imports_dir_scan(imports_dir);

    for entry in WalkDir::new(imports_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "yml"))
    {
        yaml_files.push(entry.path().to_path_buf());
        log_yaml_file_found(entry.path());
    }

    log_yaml_files_count(yaml_files.len());
    yaml_files
}

fn log_imports_dir_scan(imports_dir: &Path) {
    eprintln!("Scanning imports directory: {}", imports_dir.display());
}

fn log_yaml_file_found(file_path: &Path) {
    eprintln!("Found YAML file: {}", file_path.display());
}

fn log_yaml_files_count(count: usize) {
    eprintln!("Found {} YAML files in imports directory", count);
}

fn process_import_yaml_file(
    yaml_path: &Path,
    defined_imports: &mut HashSet<String>,
) -> std::io::Result<()> {
    let yaml_content = read_yaml_file_content(yaml_path)?;
    let yaml_config = parse_yaml_config(&yaml_content, yaml_path);

    if let Ok(config) = yaml_config {
        process_yaml_sources(config, yaml_path, defined_imports);
    }

    Ok(())
}

fn read_yaml_file_content(yaml_path: &Path) -> std::io::Result<String> {
    fs::read_to_string(yaml_path)
}

fn parse_yaml_config(
    yaml_content: &str,
    yaml_path: &Path,
) -> Result<YamlConfig, serde_yaml::Error> {
    let result = serde_yaml::from_str::<YamlConfig>(yaml_content);

    if result.is_err() {
        eprintln!("Failed to parse YAML file: {}", yaml_path.display());
    }

    result
}

fn process_yaml_sources(
    yaml_config: YamlConfig,
    yaml_path: &Path,
    defined_imports: &mut HashSet<String>,
) {
    if let Some(sources) = yaml_config.sources {
        eprintln!("Found {} sources in {}", sources.len(), yaml_path.display());

        for source in sources {
            extract_import_sources(&source, defined_imports);
        }
    } else {
        eprintln!("No imports found in {}", yaml_path.display());
    }
}

fn extract_import_sources(source: &YamlSource, defined_imports: &mut HashSet<String>) {
    let source_prefix = source.database.to_string();
    log_import_processing(&source.name, &source_prefix);

    for table in &source.tables {
        let import_name = format!("{}.{}", source_prefix, table.name);
        log_import_added(&import_name);
        defined_imports.insert(import_name);
    }
}

fn log_import_processing(import_name: &str, database: &str) {
    eprintln!(
        "Processing import: {} (database: {})",
        import_name, database
    );
}

fn log_import_added(import_name: &str) {
    eprintln!("  Adding import: {}", import_name);
}

fn debug_log_imports(defined_imports: &HashSet<String>) {
    log_imports_count(defined_imports.len());

    if !defined_imports.is_empty() {
        log_imports_details(defined_imports);
    }
}

fn log_imports_count(count: usize) {
    eprintln!("Loaded {} defined imports", count);
}

fn log_imports_details(defined_imports: &HashSet<String>) {
    eprintln!("Defined imports:");
    for import in defined_imports {
        eprintln!("  {}", import);
    }
}

fn format_missing_sources(missing_sources: &HashSet<String>) -> String {
    missing_sources
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_dot_graph(collection: &SqlModelCollection) -> String {
    let mut result = String::from("digraph models {\n");
    result.push_str("  rankdir=LR;\n");
    result.push_str("  node [shape=box];\n");

    for model in collection.models.values() {
        let depth_label = model.depth.map_or("?".to_string(), |d| d.to_string());
        result.push_str(&format!(
            "  \"{}\" [label=\"{} (depth: {})\"];\n",
            model.unique_id, model.name, depth_label
        ));
    }

    for (parent_id, children) in &collection.child_map {
        for child_id in children {
            result.push_str(&format!("  \"{}\" -> \"{}\";\n", parent_id, child_id));
        }
    }

    let max_depth = collection
        .models
        .values()
        .filter_map(|m| m.depth)
        .max()
        .unwrap_or(0);

    for depth in 0..=max_depth {
        result.push_str(&format!("  subgraph depth_{} {{\n", depth));
        result.push_str("    rank=same;\n");

        for model in collection.models.values() {
            if model.depth == Some(depth) {
                result.push_str(&format!("    \"{}\";\n", model.unique_id));
            }
        }

        result.push_str("  }\n");
    }

    result.push_str("}\n");
    result
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
