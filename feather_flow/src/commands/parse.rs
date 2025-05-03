use std::fs;
use std::path::PathBuf;

use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser as SqlParser;
use walkdir::WalkDir;

/// A model parsed from a SQL file
pub struct ParsedModel {
    /// Name of the model (filename without extension)
    pub name: String,
    /// Original parsed SQL statements
    pub parsed_statements: Vec<sqlparser::ast::Statement>,
}

/// Run the parse command
pub fn parse_command(
    model_path: &PathBuf,
    _format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Parsing SQL files in: {}", model_path.display());

    let sql_files = find_sql_files(model_path)?;
    println!("Found {} SQL files", sql_files.len());

    let mut models = Vec::new();
    for file_path in &sql_files {
        match parse_sql_file(file_path) {
            Ok(model) => {
                println!("Successfully parsed: {}", file_path.display());
                models.push(model);
            }
            Err(err) => {
                eprintln!("Error parsing {}: {}", file_path.display(), err);
            }
        }
    }

    println!(
        "Successfully parsed {} out of {} SQL files",
        models.len(),
        sql_files.len()
    );

    println!("\n--- External Dependencies ---");
    for model in &models {
        let external_deps = get_external_table_deps(&model.parsed_statements);
        println!("Model {}: {:?}", model.name, external_deps);
    }

    Ok(())
}

/// Find all SQL files in the given directory (recursively)
fn find_sql_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut sql_files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "sql" {
                    sql_files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(sql_files)
}

/// Parse a single SQL file and return a ParsedModel
fn parse_sql_file(file_path: &PathBuf) -> Result<ParsedModel, Box<dyn std::error::Error>> {
    let sql = fs::read_to_string(file_path)?;

    let name = if let Some(file_stem) = file_path.file_stem() {
        file_stem.to_string_lossy().to_string()
    } else {
        return Err("Could not extract file name".into());
    };
    // Create a dialect for parsing. Right now, we are hardcoding DuckDB.
    let dialect = DuckDbDialect {};
    let ast =
        SqlParser::parse_sql(&dialect, &sql).map_err(|e| format!("SQL parse error: {}", e))?;

    Ok(ParsedModel {
        name,
        parsed_statements: ast,
    })
}

/// Extract table names from a SQL statement, including tables from CTEs (WITH clauses)
pub fn get_table_names(statements: &[sqlparser::ast::Statement]) -> Vec<String> {
    let mut table_names = Vec::new();

    for statement in statements {
        if let sqlparser::ast::Statement::Query(query) = statement {
            // Extract tables from the main query
            extract_tables_from_query(query, &mut table_names);
        }
    }

    table_names
}

/// Extract only external table dependencies (no CTEs, no functions, qualified tables only)
/// This is useful for testing exact dependencies and for ff parse command
pub fn get_external_table_deps(statements: &[sqlparser::ast::Statement]) -> Vec<String> {
    // Get all table names
    let all_tables = get_table_names(statements);

    // Filter to only include schema-qualified tables
    all_tables
        .into_iter()
        .filter(|table| table.contains('.'))
        .collect()
}
/// Extract tables from a SQL query
pub fn extract_tables_from_query(query: &sqlparser::ast::Query, table_names: &mut Vec<String>) {
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
        sqlparser::ast::SetExpr::Select(select) => {
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
        sqlparser::ast::SetExpr::Query(subquery) => {
            extract_tables_from_query(subquery, table_names);
        }
        sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
            // For UNION, INTERSECT, EXCEPT
            extract_tables_from_set_expr(left, table_names);
            extract_tables_from_set_expr(right, table_names);
        }
        _ => {}
    }
}

/// Helper function to extract tables from a SetExpr
pub fn extract_tables_from_set_expr(expr: &sqlparser::ast::SetExpr, table_names: &mut Vec<String>) {
    match expr {
        sqlparser::ast::SetExpr::Select(select) => {
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
        sqlparser::ast::SetExpr::Query(subquery) => {
            extract_tables_from_query(subquery, table_names);
        }
        sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
            extract_tables_from_set_expr(left, table_names);
            extract_tables_from_set_expr(right, table_names);
        }
        _ => {}
    }
}

/// Extract tables from expressions (for subqueries in WHERE, etc.)
pub fn extract_tables_from_expr(expr: &sqlparser::ast::Expr, table_names: &mut Vec<String>) {
    match expr {
        sqlparser::ast::Expr::Subquery(subquery) => {
            extract_tables_from_query(subquery, table_names);
        }
        sqlparser::ast::Expr::BinaryOp { left, right, .. } => {
            extract_tables_from_expr(left, table_names);
            extract_tables_from_expr(right, table_names);
        }
        sqlparser::ast::Expr::UnaryOp { expr, .. } => {
            extract_tables_from_expr(expr, table_names);
        }
        sqlparser::ast::Expr::Cast { expr, .. } => {
            extract_tables_from_expr(expr, table_names);
        }
        sqlparser::ast::Expr::InSubquery { subquery, .. } => {
            extract_tables_from_query(subquery, table_names);
        }
        sqlparser::ast::Expr::InList { list, .. } => {
            for item in list {
                extract_tables_from_expr(item, table_names);
            }
        }
        sqlparser::ast::Expr::Function(func) => {
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

            // Process arguments to extract potential table references
            // We don't need to extract from arguments for now
        }
        sqlparser::ast::Expr::Case {
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
pub fn extract_table_from_relation(
    relation: &sqlparser::ast::TableFactor,
    table_names: &mut Vec<String>,
) {
    match relation {
        sqlparser::ast::TableFactor::Table { name, .. } => {
            // This is a direct table reference
            table_names.push(name.to_string());
        }
        sqlparser::ast::TableFactor::Derived { subquery, .. } => {
            // This is a derived table (subquery)
            extract_tables_from_query(subquery, table_names);
        }
        sqlparser::ast::TableFactor::TableFunction { expr, .. } => {
            // This is a table function (like unnest() or flatten())
            extract_tables_from_expr(expr, table_names);
        }
        sqlparser::ast::TableFactor::NestedJoin {
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
