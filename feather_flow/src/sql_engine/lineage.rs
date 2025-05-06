//! Column-level lineage tracking for SQL
//!
//! TODO: This module provides infrastructure for column-level lineage tracking,
//! which allows tracing how data flows from source columns to target columns.
//! Currently, this feature is only used in tests, but is planned to be fully
//! integrated into the main application for data lineage visualization and analysis.
use std::collections::{HashMap, HashSet};
use std::fmt;

use sqlparser::ast::{Expr, Query, SelectItem, SetExpr, Statement, TableFactor};
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;

/// Represents a column reference in a table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnRef {
    /// Table name (optional if column is ambiguous)
    pub table: Option<String>,
    /// Column name
    pub column: String,
}

impl ColumnRef {
    /// Create a new column reference
    #[allow(dead_code)]
    pub fn new(table: Option<String>, column: String) -> Self {
        Self { table, column }
    }
}

// Implement Display instead of inherent to_string
impl fmt::Display for ColumnRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.table {
            Some(table) => write!(f, "{}.{}", table, self.column),
            None => write!(f, "{}", self.column),
        }
    }
}

/// Represents column-level lineage information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ColumnLineage {
    /// Target column (in the result set)
    pub target: ColumnRef,
    /// Source columns (from source tables)
    pub sources: Vec<ColumnRef>,
    /// Transformation type (e.g., "direct", "aggregation", "expression")
    pub transformation: String,
}

/// Extract column-level lineage from SQL
#[allow(dead_code)]
pub fn extract_column_lineage(sql: &str) -> Result<Vec<ColumnLineage>, String> {
    let dialect = DuckDbDialect {};
    let statements =
        Parser::parse_sql(&dialect, sql).map_err(|e| format!("Error parsing SQL: {}", e))?;

    let mut lineage_results = Vec::new();

    for stmt in &statements {
        if let Statement::Query(query) = stmt {
            let query_lineage = extract_query_lineage(query)?;
            lineage_results.extend(query_lineage);
        }
    }

    Ok(lineage_results)
}

/// Extract column lineage from a specific query
#[allow(dead_code)]
fn extract_query_lineage(query: &Query) -> Result<Vec<ColumnLineage>, String> {
    if let SetExpr::Select(select) = &*query.body {
        // Step 1: Build a map of table aliases
        let mut alias_map = HashMap::new();

        for table_with_joins in &select.from {
            collect_table_aliases(&table_with_joins.relation, &mut alias_map);

            // Process joins
            for join in &table_with_joins.joins {
                collect_table_aliases(&join.relation, &mut alias_map);
            }
        }

        // Step 2: Process each column in the projection
        let mut lineage_results = Vec::new();

        for (idx, item) in select.projection.iter().enumerate() {
            match item {
                SelectItem::UnnamedExpr(expr) => {
                    // For expressions without explicit alias, create synthetic name
                    let target_name = match expr {
                        Expr::Identifier(ident) => ident.value.clone(),
                        Expr::CompoundIdentifier(idents) if idents.len() == 2 => {
                            idents[1].value.clone()
                        }
                        _ => format!("_col{}", idx + 1), // Synthetic name for complex expressions
                    };

                    let target = ColumnRef::new(None, target_name);
                    let sources = extract_expr_columns(expr, &alias_map, &select.from);

                    // Determine transformation type
                    let transformation = determine_transformation_type(expr);

                    lineage_results.push(ColumnLineage {
                        target,
                        sources,
                        transformation,
                    });
                }
                SelectItem::ExprWithAlias { expr, alias } => {
                    let target = ColumnRef::new(None, alias.value.clone());
                    let sources = extract_expr_columns(expr, &alias_map, &select.from);
                    let transformation = determine_transformation_type(expr);

                    lineage_results.push(ColumnLineage {
                        target,
                        sources,
                        transformation,
                    });
                }
                SelectItem::Wildcard(_) => {
                    // For * we need to expand all columns from all tables
                    // This is simplistic - in a real implementation we'd need
                    // metadata about available columns in each table
                    for table in alias_map.keys() {
                        lineage_results.push(ColumnLineage {
                            target: ColumnRef::new(Some(table.clone()), "*".to_string()),
                            sources: vec![ColumnRef::new(Some(table.clone()), "*".to_string())],
                            transformation: "direct".to_string(),
                        });
                    }
                }
                SelectItem::QualifiedWildcard(obj_name, _) => {
                    // For table.* we expand all columns from that table
                    if !obj_name.0.is_empty() {
                        let table_name = obj_name.0[0].value.clone();
                        lineage_results.push(ColumnLineage {
                            target: ColumnRef::new(Some(table_name.clone()), "*".to_string()),
                            sources: vec![ColumnRef::new(Some(table_name), "*".to_string())],
                            transformation: "direct".to_string(),
                        });
                    }
                }
            }
        }

        Ok(lineage_results)
    } else {
        // Only supporting SELECT statements for now
        Ok(vec![])
    }
}

/// Extract column references from an expression
#[allow(dead_code)]
fn extract_expr_columns(
    expr: &Expr,
    alias_map: &HashMap<String, String>,
    from_tables: &[sqlparser::ast::TableWithJoins],
) -> Vec<ColumnRef> {
    let mut columns = HashSet::new();

    match expr {
        // Column reference: col or table.col
        Expr::Identifier(ident) => {
            // Simple column reference (no table)
            // For simple column references, try to find which table it belongs to
            // For this simplified implementation, we just use the first table
            if !from_tables.is_empty() {
                if let TableFactor::Table { name, .. } = &from_tables[0].relation {
                    if !name.0.is_empty() {
                        let table_name = name.0.last().unwrap().value.clone();
                        columns.insert(ColumnRef::new(Some(table_name), ident.value.clone()));
                        return columns.into_iter().collect();
                    }
                }
            }
            columns.insert(ColumnRef::new(None, ident.value.clone()));
        }
        Expr::CompoundIdentifier(idents) if idents.len() == 2 => {
            // Table.column format
            let table_ref = idents[0].value.clone();
            let column_name = idents[1].value.clone();

            // If it's an alias, use the real table name
            let real_table = alias_map.get(&table_ref).cloned().unwrap_or(table_ref);
            columns.insert(ColumnRef::new(Some(real_table), column_name));
        }
        // Binary operations (e.g., a + b, a > b)
        Expr::BinaryOp { left, right, .. } => {
            let left_columns = extract_expr_columns(left, alias_map, from_tables);
            let right_columns = extract_expr_columns(right, alias_map, from_tables);

            columns.extend(left_columns);
            columns.extend(right_columns);
        }
        // Function calls (e.g., SUM(a), COUNT(*))
        Expr::Function(func) => {
            // Simply check the function name
            if !func.name.0.is_empty() {
                let func_name = func.name.0[0].value.to_lowercase();
                if func_name == "count" {
                    // COUNT is usually special, but for simplicity we'll just skip it
                    // In a real implementation, we'd need to extract columns from the args
                } else {
                    // For now, we don't extract columns from function arguments
                    // This is a limitation of the current implementation
                }
            }
        }
        // Handle other expression types as needed
        _ => {}
    }

    columns.into_iter().collect()
}

/// Collect table aliases from a TableFactor
#[allow(dead_code)]
fn collect_table_aliases(table_factor: &TableFactor, alias_map: &mut HashMap<String, String>) {
    match table_factor {
        TableFactor::Table { name, alias, .. } => {
            // Get the table name from the ObjectName's last element in the vector
            if !name.0.is_empty() {
                let real_table = name.0.last().unwrap().value.clone();

                // If there's an alias, map it to the real table name
                if let Some(table_alias) = alias {
                    alias_map.insert(table_alias.name.value.clone(), real_table.clone());
                }

                // Also map the real name to itself
                alias_map.insert(real_table.clone(), real_table);
            }
        }
        // Handle other table factor types as needed
        _ => {}
    }
}

/// Determine the transformation type
#[allow(dead_code)]
fn determine_transformation_type(expr: &Expr) -> String {
    match expr {
        // Direct column reference
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => "direct".to_string(),

        // Function calls typically indicate aggregation or transformation
        Expr::Function(func) => {
            if !func.name.0.is_empty() {
                let func_name = func.name.0[0].value.to_lowercase();
                if ["sum", "count", "avg", "min", "max"].contains(&func_name.as_str()) {
                    "aggregation".to_string()
                } else {
                    "function".to_string()
                }
            } else {
                "function".to_string()
            }
        }

        // Binary operations indicate expressions
        Expr::BinaryOp { .. } => "expression".to_string(),

        // Case expressions
        Expr::Case { .. } => "case_when".to_string(),

        // Cast expressions
        Expr::Cast { .. } => "cast".to_string(),

        // Default for other types
        _ => "unknown".to_string(),
    }
}

/// Generate a graph representation of the lineage (dot format for Graphviz)
#[allow(dead_code)]
pub fn generate_lineage_graph(lineage: &[ColumnLineage]) -> String {
    let mut result = String::from("digraph lineage {\n");
    result.push_str("  rankdir=LR;\n");
    result.push_str("  node [shape=box];\n");

    // Add nodes and edges
    for item in lineage {
        let target_name = item.target.to_string();

        // Add target node
        result.push_str(&format!(
            "  \"{}\" [style=filled, fillcolor=lightblue];\n",
            target_name
        ));

        // Add source nodes and edges
        for source in &item.sources {
            let source_name = source.to_string();
            result.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                source_name, target_name, item.transformation
            ));
        }
    }

    result.push_str("}\n");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let sql = "SELECT id, name FROM users";

        let lineage: Vec<ColumnLineage> = extract_column_lineage(sql).unwrap();
        assert_eq!(lineage.len(), 2);

        assert_eq!(lineage[0].target.column, "id");
        assert_eq!(lineage[0].sources[0].table, Some("users".to_string()));
        assert_eq!(lineage[0].sources[0].column, "id");
        assert_eq!(lineage[0].transformation, "direct");

        assert_eq!(lineage[1].target.column, "name");
        assert_eq!(lineage[1].sources[0].table, Some("users".to_string()));
        assert_eq!(lineage[1].sources[0].column, "name");
        assert_eq!(lineage[1].transformation, "direct");
    }

    #[test]
    fn test_with_alias() {
        let sql = "SELECT u.id, u.name as user_name FROM users u";

        let lineage = extract_column_lineage(sql).unwrap();
        assert_eq!(lineage.len(), 2);

        assert_eq!(lineage[0].target.column, "id");
        assert_eq!(lineage[0].sources[0].table, Some("users".to_string()));
        assert_eq!(lineage[0].sources[0].column, "id");

        assert_eq!(lineage[1].target.column, "user_name");
        assert_eq!(lineage[1].sources[0].table, Some("users".to_string()));
        assert_eq!(lineage[1].sources[0].column, "name");
    }

    #[test]
    fn test_with_expression() {
        let sql = "SELECT id, price * quantity as total FROM orders";

        let lineage = extract_column_lineage(sql).unwrap();
        assert_eq!(lineage.len(), 2);

        assert_eq!(lineage[0].target.column, "id");
        assert_eq!(lineage[0].transformation, "direct");

        assert_eq!(lineage[1].target.column, "total");
        assert_eq!(lineage[1].transformation, "expression");
        assert_eq!(lineage[1].sources.len(), 2);

        // Verify that both columns are referenced
        let source_columns: Vec<String> = lineage[1]
            .sources
            .iter()
            .map(|s| s.column.clone())
            .collect();
        assert!(source_columns.contains(&"price".to_string()));
        assert!(source_columns.contains(&"quantity".to_string()));
    }

    #[test]
    fn test_with_join() {
        let sql = "SELECT c.id, c.name, o.order_date 
                   FROM customers c 
                   JOIN orders o ON c.id = o.customer_id";

        let lineage = extract_column_lineage(sql).unwrap();
        assert_eq!(lineage.len(), 3);

        // Check first column lineage
        assert_eq!(lineage[0].target.column, "id");
        assert_eq!(lineage[0].sources[0].table, Some("customers".to_string()));
        assert_eq!(lineage[0].sources[0].column, "id");

        // Check second column lineage
        assert_eq!(lineage[1].target.column, "name");
        assert_eq!(lineage[1].sources[0].table, Some("customers".to_string()));
        assert_eq!(lineage[1].sources[0].column, "name");

        // Check third column lineage
        assert_eq!(lineage[2].target.column, "order_date");
        assert_eq!(lineage[2].sources[0].table, Some("orders".to_string()));
        assert_eq!(lineage[2].sources[0].column, "order_date");
    }

    #[test]
    fn test_with_aggregation() {
        let sql = "SELECT 
                     customer_id, 
                     COUNT(*) as order_count, 
                     SUM(amount) as total_amount 
                   FROM orders 
                   GROUP BY customer_id";

        let lineage = extract_column_lineage(sql).unwrap();
        assert_eq!(lineage.len(), 3);

        // Check customer_id lineage
        assert_eq!(lineage[0].target.column, "customer_id");
        assert_eq!(lineage[0].transformation, "direct");

        // Check order_count lineage
        assert_eq!(lineage[1].target.column, "order_count");
        assert_eq!(lineage[1].transformation, "aggregation");

        // Check total_amount lineage
        assert_eq!(lineage[2].target.column, "total_amount");
        assert_eq!(lineage[2].transformation, "aggregation");
    }
}
