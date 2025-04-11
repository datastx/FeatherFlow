use std::collections::HashMap;
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

    // Build the dependency graph
    let model_graph = build_dependency_graph(&models);

    // Check for cycles
    if let Err(cycle_error) = check_for_cycles(&model_graph) {
        println!("⚠️ Warning: {}", cycle_error);
    } else {
        println!("✅ No circular dependencies found");
    }

    println!("\n--- Generating Model Dependency Graph ---");
    match format {
        "dot" => println!("{}", generate_dot_graph(&model_graph)),
        "json" => println!("{}", generate_json_graph(&model_graph)),
        "text" => print_text_graph(&model_graph),
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
    // Read the SQL file
    let sql = fs::read_to_string(file_path)?;

    // Extract the model name from the file path
    let name = if let Some(file_stem) = file_path.file_stem() {
        file_stem.to_string_lossy().to_string()
    } else {
        return Err("Could not extract file name".into());
    };

    // Parse the SQL to extract table references
    let dialect = DuckDbDialect {};
    let ast =
        SqlParser::parse_sql(&dialect, &sql).map_err(|e| format!("SQL parse error: {}", e))?;

    // Extract table references
    let referenced_tables = get_table_names(&ast);

    Ok(ParsedModel {
        path: file_path.clone(),
        name,
        referenced_tables,
    })
}

/// Extract table names from a SQL statement
fn get_table_names(statements: &[sqlparser::ast::Statement]) -> Vec<String> {
    let mut table_names = Vec::new();

    for statement in statements {
        if let sqlparser::ast::Statement::Query(query) = statement {
            if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
                for table_with_joins in &select.from {
                    collect_table_names(&table_with_joins.relation, &mut table_names);
                    for join in &table_with_joins.joins {
                        collect_table_names(&join.relation, &mut table_names);
                    }
                }
            }
        }
    }

    table_names
}

/// Helper function to collect table names from a table factor
fn collect_table_names(table_factor: &sqlparser::ast::TableFactor, table_names: &mut Vec<String>) {
    if let sqlparser::ast::TableFactor::Table { name, .. } = table_factor {
        table_names.push(name.to_string());
    }
}

/// Build a dependency graph from parsed models
fn build_dependency_graph(models: &[ParsedModel]) -> HashMap<String, Vec<String>> {
    let mut model_map: HashMap<String, &ParsedModel> = HashMap::new();

    for model in models {
        model_map.insert(model.name.clone(), model);
    }

    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for model in models {
        let mut dependencies = Vec::new();

        for table in &model.referenced_tables {
            let table_parts: Vec<&str> = table.split('.').collect();
            let table_name = if table_parts.len() > 1 {
                table_parts[1]
            } else {
                table_parts[0]
            };

            if model_map.contains_key(table_name) && table_name != model.name {
                dependencies.push(table_name.to_string());
            }
        }

        graph.insert(model.name.clone(), dependencies);
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
