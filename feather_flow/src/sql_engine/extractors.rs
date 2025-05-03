//! Table and dependency extraction utilities for SQL

use sqlparser::ast::{Expr, Query, SetExpr, Statement, TableFactor};
use std::collections::HashSet;

/// Extract table names from a SQL statement, including tables from CTEs (WITH clauses)
pub fn get_table_names(statements: &[Statement]) -> Vec<String> {
    let mut table_names = Vec::new();

    for statement in statements {
        if let Statement::Query(query) = statement {
            // Extract tables from the main query
            extract_tables_from_query(query, &mut table_names);
        }
    }

    table_names
}

/// Extract only external table dependencies (no CTEs, no functions, qualified tables only)
pub fn get_external_table_deps(statements: &[Statement]) -> Vec<String> {
    // Get all table names
    let all_tables = get_table_names(statements);

    // Filter to only include schema-qualified tables
    all_tables
        .into_iter()
        .filter(|table| table.contains('.'))
        .collect()
}

/// Extract tables from a SQL query
pub fn extract_tables_from_query(query: &Query, table_names: &mut Vec<String>) {
    // Extract tables from CTEs (WITH clause) first
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            // Record the CTE name itself
            table_names.push(cte.alias.name.value.clone());

            // Extract tables from the CTE definition
            extract_tables_from_query(&cte.query, table_names);
        }
    }

    // Extract tables from the query body
    match &*query.body {
        SetExpr::Select(select) => {
            // Extract tables from FROM clause
            for table_with_joins in &select.from {
                extract_table_from_relation(&table_with_joins.relation, table_names);

                // Extract tables from JOINs
                for join in &table_with_joins.joins {
                    extract_table_from_relation(&join.relation, table_names);
                }
            }

            // Extract tables from WHERE clause (for subqueries)
            if let Some(where_expr) = &select.selection {
                extract_tables_from_expr(where_expr, table_names);
            }

            // Extract tables from SELECT expressions (for subqueries)
            for item in &select.projection {
                match item {
                    sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => {
                        extract_tables_from_expr(expr, table_names);
                    }
                    sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                        extract_tables_from_expr(expr, table_names);
                    }
                    _ => {}
                }
            }

            // Extract tables from GROUP BY, HAVING, etc.
            if let Some(having) = &select.having {
                extract_tables_from_expr(having, table_names);
            }
        }
        SetExpr::Query(subquery) => {
            extract_tables_from_query(subquery, table_names);
        }
        SetExpr::SetOperation { left, right, .. } => {
            // For UNION, INTERSECT, EXCEPT
            extract_tables_from_set_expr(left, table_names);
            extract_tables_from_set_expr(right, table_names);
        }
        _ => {}
    }
}

/// Helper function to extract tables from a SetExpr
pub fn extract_tables_from_set_expr(expr: &SetExpr, table_names: &mut Vec<String>) {
    match expr {
        SetExpr::Select(select) => {
            // Extract tables from FROM clause
            for table_with_joins in &select.from {
                extract_table_from_relation(&table_with_joins.relation, table_names);
                for join in &table_with_joins.joins {
                    extract_table_from_relation(&join.relation, table_names);
                }
            }

            // Process subqueries in WHERE
            if let Some(where_expr) = &select.selection {
                extract_tables_from_expr(where_expr, table_names);
            }
        }
        SetExpr::Query(subquery) => {
            extract_tables_from_query(subquery, table_names);
        }
        SetExpr::SetOperation { left, right, .. } => {
            extract_tables_from_set_expr(left, table_names);
            extract_tables_from_set_expr(right, table_names);
        }
        _ => {}
    }
}

/// Extract tables from expressions (for subqueries in WHERE, etc.)
pub fn extract_tables_from_expr(expr: &Expr, table_names: &mut Vec<String>) {
    match expr {
        Expr::Subquery(subquery) => {
            extract_tables_from_query(subquery, table_names);
        }
        Expr::BinaryOp { left, right, .. } => {
            extract_tables_from_expr(left, table_names);
            extract_tables_from_expr(right, table_names);
        }
        Expr::UnaryOp { expr, .. } => {
            extract_tables_from_expr(expr, table_names);
        }
        Expr::Cast { expr, .. } => {
            extract_tables_from_expr(expr, table_names);
        }
        Expr::InSubquery { subquery, .. } => {
            extract_tables_from_query(subquery, table_names);
        }
        Expr::InList { list, .. } => {
            for item in list {
                extract_tables_from_expr(item, table_names);
            }
        }
        Expr::Function(func) => {
            // Skip common SQL aggregation and scalar functions
            let common_sql_functions = [
                "COUNT",
                "SUM",
                "AVG",
                "MIN",
                "MAX",
                "DATE",
                "TIME",
                "TIMESTAMP",
                "EXTRACT",
                "CONCAT",
                "SUBSTRING",
                "UPPER",
                "LOWER",
                "COALESCE",
                "NULLIF",
                "CAST",
                "CONVERT",
                "ROUND",
                "FLOOR",
                "CEILING",
                "ABS",
                "DATE_TRUNC",
                "DATE_PART",
                "DATE_DIFF",
                "DATE_ADD",
                "DATE_SUB",
                "CURRENT_DATE",
                "CURRENT_TIME",
                "CURRENT_TIMESTAMP",
                "CASE",
                "IF",
                "IFNULL",
                "NVL",
                "IIF",
            ];

            let func_name = func.name.to_string().to_uppercase();
            if !common_sql_functions.contains(&func_name.as_str()) {
                table_names.push(func.name.to_string());
            }
        }
        Expr::Case {
            operand,
            conditions,
            results,
            else_result,
            ..
        } => {
            if let Some(op) = operand {
                extract_tables_from_expr(op, table_names);
            }
            for condition in conditions {
                extract_tables_from_expr(condition, table_names);
            }
            for result in results {
                extract_tables_from_expr(result, table_names);
            }
            if let Some(else_res) = else_result {
                extract_tables_from_expr(else_res, table_names);
            }
        }
        // Skip other expression types for now
        _ => {}
    }
}

/// Helper function to extract table names from a relation
pub fn extract_table_from_relation(relation: &TableFactor, table_names: &mut Vec<String>) {
    match relation {
        TableFactor::Table { name, .. } => {
            // This is a direct table reference
            table_names.push(name.to_string());
        }
        TableFactor::Derived { subquery, .. } => {
            // This is a derived table (subquery)
            extract_tables_from_query(subquery, table_names);
        }
        TableFactor::TableFunction { expr, .. } => {
            // This is a table function (like unnest() or flatten())
            extract_tables_from_expr(expr, table_names);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            // This is a nested join
            extract_table_from_relation(&table_with_joins.relation, table_names);
            for join in &table_with_joins.joins {
                extract_table_from_relation(&join.relation, table_names);
            }
        }
        // Skip other table factor types for now
        _ => {}
    }
}

/// Get all external table dependencies as a HashSet
pub fn get_external_table_deps_set(statements: &[Statement]) -> HashSet<String> {
    get_external_table_deps(statements).into_iter().collect()
}
