use std::collections::HashMap;

/// Represents a SQL table schema
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<Vec<String>>,
}

/// Represents a column definition
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: SqlType,
    pub nullable: bool,
}

/// Represents SQL data types
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SqlType {
    Integer,
    Float,
    Text,
    Boolean,
    Date,
    Timestamp,
    // Other types can be added as needed
}

/// Table manager for handling table operations
#[derive(Default)]
#[allow(dead_code)]
pub struct TableManager {
    schemas: HashMap<String, TableSchema>,
}

impl TableManager {
    /// Create a new empty table manager
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Add or update a table schema
    #[allow(dead_code)]
    pub fn register_schema(&mut self, schema: TableSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Get all available table names
    #[allow(dead_code)]
    pub fn get_table_names(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// Get a specific table schema by name
    #[allow(dead_code)]
    pub fn get_schema(&self, table_name: &str) -> Option<&TableSchema> {
        self.schemas.get(table_name)
    }

    /// Get column names for a specific table
    #[allow(dead_code)]
    pub fn get_column_names(&self, table_name: &str) -> Option<Vec<String>> {
        self.get_schema(table_name)
            .map(|schema| schema.columns.iter().map(|col| col.name.clone()).collect())
    }
}
