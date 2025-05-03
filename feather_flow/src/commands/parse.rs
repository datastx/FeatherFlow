use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser as SqlParser;
use walkdir::WalkDir;

/// A model parsed from a SQL file
pub struct ParsedModel {
    /// Path to the model file
    #[allow(dead_code)]
    pub path: PathBuf,
    /// Name of the model (filename without extension)
    pub name: String,
    /// Tables referenced in the SQL
    pub referenced_tables: Vec<String>,
    /// Original parsed SQL statements
    pub parsed_statements: Vec<sqlparser::ast::Statement>,
}

/// Run the parse command
pub fn parse_command(model_path: &PathBuf, format: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    // Build the dependency graph (only including internal models)
    let internal_model_graph = build_dependency_graph(&models, false);

    // Build the full dependency graph (including external tables)
    let full_model_graph = build_dependency_graph(&models, true);

    // Check for cycles (only with internal models)
    if let Err(cycle_error) = check_for_cycles(&internal_model_graph) {
        println!("⚠️ Warning: {}", cycle_error);
    } else {
        println!("✅ No circular dependencies found");
    }

    println!("\n--- Generating Model Dependency Graph ---");
    match format {
        "dot" => println!("{}", generate_dot_graph(&full_model_graph)),
        "json" => println!("{}", generate_json_graph(&full_model_graph)),
        "text" => print_text_graph(&full_model_graph),
        _ => eprintln!("Unsupported graph format: {}", format),
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

    let referenced_tables = get_table_names(&ast);

    Ok(ParsedModel {
        path: file_path.clone(),
        name,
        referenced_tables,
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
/// This is useful for testing exact dependencies
pub fn get_external_table_deps(statements: &[sqlparser::ast::Statement]) -> Vec<String> {
    // Collect all CTEs from all statements
    let mut all_ctes = HashSet::new();
    
    // First, collect all CTEs defined in all statements
    for statement in statements {
        if let sqlparser::ast::Statement::Query(query) = statement {
            if let Some(with) = &query.with {
                for cte in &with.cte_tables {
                    all_ctes.insert(cte.alias.name.value.clone());
                }
            }
        }
    }
    
    // Common SQL functions to exclude
    let common_sql_functions = [
        "COUNT", "SUM", "AVG", "MIN", "MAX", 
        "DATE", "TIME", "TIMESTAMP", "EXTRACT",
        "CONCAT", "SUBSTRING", "UPPER", "LOWER",
        "COALESCE", "NULLIF", "CAST", "CONVERT",
        "ROUND", "FLOOR", "CEILING", "ABS",
        "DATE_TRUNC", "DATE_PART", "DATE_DIFF", "DATE_ADD", "DATE_SUB",
        "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP",
        "CASE", "IF", "IFNULL", "NVL", "IIF",
    ];
    
    // Now extract tables directly to avoid nested CTEs issue
    let mut result = Vec::new();
    let mut external_tables = HashSet::new();
    
    // Process each statement individually
    for statement in statements {
        if let sqlparser::ast::Statement::Query(query) = statement {
            extract_external_tables(
                query, 
                &mut external_tables, 
                &all_ctes, 
                &common_sql_functions
            );
        }
    }
    
    for table in external_tables {
        result.push(table);
    }
    
    result
}

/// Helper function to extract external tables (recursive helper for get_external_table_deps)
fn extract_external_tables(
    query: &sqlparser::ast::Query,
    external_tables: &mut HashSet<String>,
    all_ctes: &HashSet<String>,
    common_functions: &[&str],
) {
    // Process query body
    match &*query.body {
        sqlparser::ast::SetExpr::Select(select) => {
            // Process FROM clause
            for table_with_joins in &select.from {
                // Process main table
                if let sqlparser::ast::TableFactor::Table { name, .. } = &table_with_joins.relation {
                    let table_name = name.to_string();
                    if !all_ctes.contains(&table_name) && 
                       !common_functions.contains(&table_name.to_uppercase().as_str()) &&
                       table_name.contains('.') { // Only include fully-qualified tables
                        external_tables.insert(table_name);
                    }
                } else {
                    // For other table types (derived tables, etc.)
                    extract_external_table_from_relation(
                        &table_with_joins.relation, 
                        external_tables, 
                        all_ctes, 
                        common_functions
                    );
                }
                
                // Process JOINS
                for join in &table_with_joins.joins {
                    if let sqlparser::ast::TableFactor::Table { name, .. } = &join.relation {
                        let table_name = name.to_string();
                        if !all_ctes.contains(&table_name) && 
                           !common_functions.contains(&table_name.to_uppercase().as_str()) &&
                           table_name.contains('.') { // Only include fully-qualified tables
                            external_tables.insert(table_name);
                        }
                    } else {
                        // For other table types (derived tables, etc.)
                        extract_external_table_from_relation(
                            &join.relation, 
                            external_tables, 
                            all_ctes, 
                            common_functions
                        );
                    }
                }
            }
            
            // Extract tables from WHERE clause (for subqueries)
            if let Some(where_expr) = &select.selection {
                extract_external_tables_from_expr(where_expr, external_tables, all_ctes, common_functions);
            }
            
            // Extract tables from SELECT expressions (for subqueries)
            for item in &select.projection {
                match item {
                    sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => {
                        extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
                    }
                    sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                        extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
                    }
                    _ => {}
                }
            }
            
            // Extract tables from GROUP BY, HAVING, etc.
            if let Some(having) = &select.having {
                extract_external_tables_from_expr(having, external_tables, all_ctes, common_functions);
            }
        },
        sqlparser::ast::SetExpr::Query(subquery) => {
            extract_external_tables(subquery, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
            // Process both sides of set operation
            extract_external_tables_from_set_expr(left, external_tables, all_ctes, common_functions);
            extract_external_tables_from_set_expr(right, external_tables, all_ctes, common_functions);
        },
        _ => {}
    }
    
    // Process WITH clause (for nested CTEs)
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            extract_external_tables(&cte.query, external_tables, all_ctes, common_functions);
        }
    }
}

/// Helper function to extract external tables from relation
fn extract_external_table_from_relation(
    relation: &sqlparser::ast::TableFactor,
    external_tables: &mut HashSet<String>,
    all_ctes: &HashSet<String>,
    common_functions: &[&str],
) {
    match relation {
        sqlparser::ast::TableFactor::Table { name, .. } => {
            let table_name = name.to_string();
            if !all_ctes.contains(&table_name) && 
               !common_functions.contains(&table_name.to_uppercase().as_str()) &&
               table_name.contains('.') { // Only include fully-qualified tables
                external_tables.insert(table_name);
            }
        },
        sqlparser::ast::TableFactor::Derived { subquery, .. } => {
            // This is a derived table (subquery)
            extract_external_tables(subquery, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::TableFactor::TableFunction { expr, .. } => {
            // This is a table function
            extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::TableFactor::NestedJoin { table_with_joins, .. } => {
            // This is a nested join
            extract_external_table_from_relation(&table_with_joins.relation, external_tables, all_ctes, common_functions);
            for join in &table_with_joins.joins {
                extract_external_table_from_relation(&join.relation, external_tables, all_ctes, common_functions);
            }
        },
        _ => {},
    }
}

/// Helper function to extract external tables from a SetExpr
fn extract_external_tables_from_set_expr(
    expr: &sqlparser::ast::SetExpr,
    external_tables: &mut HashSet<String>,
    all_ctes: &HashSet<String>,
    common_functions: &[&str],
) {
    match expr {
        sqlparser::ast::SetExpr::Select(select) => {
            // Process FROM clause
            for table_with_joins in &select.from {
                extract_external_table_from_relation(
                    &table_with_joins.relation, 
                    external_tables, 
                    all_ctes, 
                    common_functions
                );
                
                for join in &table_with_joins.joins {
                    extract_external_table_from_relation(
                        &join.relation, 
                        external_tables, 
                        all_ctes, 
                        common_functions
                    );
                }
            }
            
            // Extract from WHERE clause
            if let Some(where_expr) = &select.selection {
                extract_external_tables_from_expr(where_expr, external_tables, all_ctes, common_functions);
            }
            
            // Extract tables from SELECT expressions (for subqueries)
            for item in &select.projection {
                match item {
                    sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => {
                        extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
                    }
                    sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                        extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
                    }
                    _ => {}
                }
            }
            
            // Extract tables from GROUP BY, HAVING, etc.
            if let Some(having) = &select.having {
                extract_external_tables_from_expr(having, external_tables, all_ctes, common_functions);
            }
        },
        sqlparser::ast::SetExpr::Query(subquery) => {
            extract_external_tables(subquery, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
            extract_external_tables_from_set_expr(left, external_tables, all_ctes, common_functions);
            extract_external_tables_from_set_expr(right, external_tables, all_ctes, common_functions);
        },
        _ => {},
    }
}

/// Extract external tables from expressions
fn extract_external_tables_from_expr(
    expr: &sqlparser::ast::Expr,
    external_tables: &mut HashSet<String>,
    all_ctes: &HashSet<String>,
    common_functions: &[&str],
) {
    match expr {
        sqlparser::ast::Expr::Subquery(subquery) => {
            extract_external_tables(subquery, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::Expr::BinaryOp { left, right, .. } => {
            extract_external_tables_from_expr(left, external_tables, all_ctes, common_functions);
            extract_external_tables_from_expr(right, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::Expr::UnaryOp { expr, .. } => {
            extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::Expr::Cast { expr, .. } => {
            extract_external_tables_from_expr(expr, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::Expr::InSubquery { subquery, .. } => {
            extract_external_tables(subquery, external_tables, all_ctes, common_functions);
        },
        sqlparser::ast::Expr::InList { list, .. } => {
            for item in list {
                extract_external_tables_from_expr(item, external_tables, all_ctes, common_functions);
            }
        },
        sqlparser::ast::Expr::Function(func) => {
            let func_name = func.name.to_string().to_uppercase();
            if !common_functions.contains(&func_name.as_str()) {
                // For functions that could represent table references, add to external_tables
                let table_name = func.name.to_string();
                if table_name.contains('.') {
                    external_tables.insert(table_name);
                }
                
                // We don't process function arguments for this version of sqlparser
            }
        },
        sqlparser::ast::Expr::Case { operand, conditions, results, else_result, .. } => {
            if let Some(op) = operand {
                extract_external_tables_from_expr(op, external_tables, all_ctes, common_functions);
            }
            for condition in conditions {
                extract_external_tables_from_expr(condition, external_tables, all_ctes, common_functions);
            }
            for result in results {
                extract_external_tables_from_expr(result, external_tables, all_ctes, common_functions);
            }
            if let Some(else_res) = else_result {
                extract_external_tables_from_expr(else_res, external_tables, all_ctes, common_functions);
            }
        },
        _ => {},
    }
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

/// Build a dependency graph from parsed models
///
/// If `include_external` is true, include all referenced tables, even those not in the model_map.
/// If `include_external` is false, only include tables that match other model names.
fn build_dependency_graph(
    models: &[ParsedModel],
    include_external: bool,
) -> HashMap<String, Vec<String>> {
    let mut model_map: HashMap<String, &ParsedModel> = HashMap::new();

    for model in models {
        model_map.insert(model.name.clone(), model);
    }

    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for model in models {
        // Find all the CTEs defined in this model
        let mut cte_names = HashSet::new();
        for stmt in &model.parsed_statements {
            if let sqlparser::ast::Statement::Query(query) = stmt {
                if let Some(with) = &query.with {
                    for cte in &with.cte_tables {
                        cte_names.insert(cte.alias.name.value.clone());
                    }
                }
            }
        }

        let mut dependencies = HashSet::new();

        for table in &model.referenced_tables {
            // Keep the original table reference for external dependencies
            let original_table = table.clone();

            // Extract the table name without schema
            let table_parts: Vec<&str> = table.split('.').collect();
            let table_name = if table_parts.len() > 1 {
                table_parts[1]
            } else {
                table_parts[0]
            };

            // Skip references to CTEs defined in this model
            if cte_names.contains(table) {
                continue;
            }

            // Determine if this is an internal model dependency or external table
            let is_internal_model = model_map.contains_key(table_name) && table_name != model.name;
            let is_external_table = !is_internal_model;

            if is_internal_model || (include_external && is_external_table) {
                // For internal models, reference by the model name
                if is_internal_model {
                    dependencies.insert(table_name.to_string());
                } else {
                    // For external tables, use the full reference including schema
                    dependencies.insert(original_table);
                }
            }
        }

        graph.insert(model.name.clone(), dependencies.into_iter().collect());
    }

    graph
}

/// Check for cycles in the dependency graph
fn check_for_cycles(graph: &HashMap<String, Vec<String>>) -> Result<(), String> {
    for start_node in graph.keys() {
        let mut visited = HashMap::new();
        for node in graph.keys() {
            visited.insert(node.clone(), false);
        }

        if has_cycle(graph, start_node, &mut visited, &mut Vec::new()) {
            return Err(format!(
                "Circular dependency detected starting from model '{}'",
                start_node
            ));
        }
    }

    Ok(())
}

/// Helper function for cycle detection
fn has_cycle(
    graph: &HashMap<String, Vec<String>>,
    current: &str,
    visited: &mut HashMap<String, bool>,
    path: &mut Vec<String>,
) -> bool {
    visited.insert(current.to_string(), true);
    path.push(current.to_string());

    if let Some(dependencies) = graph.get(current) {
        for dep in dependencies {
            // If dependency is in current path, we have a cycle
            if path.contains(dep) {
                return true;
            }

            if !visited[dep] && has_cycle(graph, dep, visited, path) {
                return true;
            }
        }
    }

    path.pop();
    false
}

/// Generate a DOT graph representation
fn generate_dot_graph(graph: &HashMap<String, Vec<String>>) -> String {
    let mut dot = String::from("digraph G {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n\n");

    for model in graph.keys() {
        dot.push_str(&format!("  \"{}\";\n", model));
    }
    dot.push('\n');

    dot.push('\n');
    for (model, dependencies) in graph {
        for dep in dependencies {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", dep, model));
        }
    }

    dot.push_str("}\n");
    dot
}

/// Generate a JSON graph representation
fn generate_json_graph(graph: &HashMap<String, Vec<String>>) -> String {
    use std::collections::BTreeMap;

    let ordered_graph: BTreeMap<&String, &Vec<String>> = graph.iter().collect();

    let mut json = String::from("{\n");

    for (i, (model, dependencies)) in ordered_graph.iter().enumerate() {
        json.push_str(&format!("  \"{}\": [", model));

        for (j, dep) in dependencies.iter().enumerate() {
            if j > 0 {
                json.push_str(", ");
            }
            json.push_str(&format!("\"{}\"", dep));
        }

        json.push(']');

        if i < ordered_graph.len() - 1 {
            json.push_str(",\n");
        } else {
            json.push('\n');
        }
    }

    json.push_str("}\n");
    json
}

/// Print a text representation of the graph
fn print_text_graph(graph: &HashMap<String, Vec<String>>) {
    use std::collections::BTreeMap;

    // Convert to BTreeMap for consistent ordering
    let ordered_graph: BTreeMap<&String, &Vec<String>> = graph.iter().collect();

    println!("Model Dependency Graph:");
    println!("======================");

    for (model, dependencies) in ordered_graph {
        println!("Model: {}", model);

        if dependencies.is_empty() {
            println!("  No dependencies");
        } else {
            println!("  Depends on:");
            for dep in dependencies {
                println!("    - {}", dep);
            }
        }
        println!();
    }
}
