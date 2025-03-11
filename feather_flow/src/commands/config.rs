use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Project configuration similar to dbt_project.yaml
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatherFlowConfig {
    /// Name of the project
    pub name: String,
    
    /// Project version
    pub version: String,
    
    /// Configuration profile (default, dev, prod, etc.)
    #[serde(default = "default_profile")]
    pub profile: String,
    
    /// Path to models directory (relative to project root)
    #[serde(default = "default_models_path")]
    pub models_path: String,
    
    /// Model-specific configurations
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,
    
    /// Additional project configurations
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Configuration for model directories
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Whether to materialize this model
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Default materialization strategy
    #[serde(default = "default_materialization")]
    pub materialized: String,
    
    /// Schema prefix to apply
    #[serde(default)]
    pub schema: Option<String>,
    
    /// Additional model-specific configurations
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

fn default_profile() -> String {
    "default".to_string()
}

fn default_models_path() -> String {
    "models".to_string()
}

fn default_true() -> bool {
    true
}

fn default_materialization() -> String {
    "view".to_string()
}

impl Default for FeatherFlowConfig {
    fn default() -> Self {
        Self {
            name: "featherflow_project".to_string(),
            version: "1.0.0".to_string(),
            profile: default_profile(),
            models_path: default_models_path(),
            models: HashMap::new(),
            extra: HashMap::new(),
        }
    }
}

/// Reads the configuration file from the specified path or looks for
/// featherflow_project.yaml in the current directory
pub fn read_config(config_path: Option<PathBuf>) -> Result<FeatherFlowConfig, Box<dyn std::error::Error>> {
    let config_path = if let Some(path) = config_path {
        path
    } else {
        // Look for config in the current directory
        let current_dir = std::env::current_dir()?;
        current_dir.join("featherflow_project.yaml")
    };

    if !config_path.exists() {
        return Err(format!("Configuration file not found at: {}", config_path.display()).into());
    }

    let config_str = std::fs::read_to_string(config_path)?;
    let config: FeatherFlowConfig = serde_yaml::from_str(&config_str)?;
    
    Ok(config)
}