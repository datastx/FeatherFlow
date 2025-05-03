//! Tests for SQL parsing functionality

use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser as SqlParser;
use std::collections::HashSet;

use crate::commands::parse::{get_external_table_deps, get_table_names};

/// Test fixture that maps SQL file paths to their expected dependencies
struct TestFixture {
    /// Path to the SQL file (relative to demo_project/models)
    path: &'static str,
    /// Expected dependencies for this model
    expected_dependencies: Vec<&'static str>,
}

/// Read a SQL file and parse its dependencies
fn parse_dependencies(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Read the SQL file
    let full_path = format!(
        "/workspaces/FeatherFlow/feather_flow/demo_project/models/{}",
        file_path
    );
    let sql = std::fs::read_to_string(full_path)?;

    // Parse SQL to AST
    let dialect = DuckDbDialect {};
    let ast =
        SqlParser::parse_sql(&dialect, &sql).map_err(|e| format!("SQL parse error: {}", e))?;

    // Extract only external table dependencies (no CTEs or functions)
    let table_refs = get_external_table_deps(&ast);

    Ok(table_refs)
}

/// Convert list of dependencies to a normalized set for comparison
fn normalize_dependencies(deps: &[String]) -> HashSet<String> {
    let mut normalized = HashSet::new();

    for dep in deps {
        // Split schema.table into parts
        let parts: Vec<&str> = dep.split('.').collect();
        if parts.len() > 1 {
            // Store both the full reference and just the table name
            normalized.insert(dep.clone());
            normalized.insert(parts[1].to_string());
        } else {
            normalized.insert(dep.clone());
        }
    }

    normalized
}

/// Check if all expected dependencies are found in the actual dependencies
fn check_dependencies(
    actual: &[String],
    expected: &[&str],
    model_path: &str,
) -> Result<(), String> {
    let actual_set = normalize_dependencies(actual);
    let expected_set: HashSet<String> = expected.iter().map(|&s| s.to_string()).collect();

    // Check if all expected dependencies are in the actual set
    let missing: Vec<_> = expected_set
        .iter()
        .filter(|dep| !actual_set.contains(*dep))
        .collect();

    if !missing.is_empty() {
        return Err(format!(
            "Model {} is missing dependencies: {:?}. Found: {:?}",
            model_path, missing, actual_set
        ));
    }

    Ok(())
}

/// Test cases for all demo project models
fn get_test_fixtures() -> Vec<TestFixture> {
    vec![
        // Human revieiewed
        TestFixture {
            path: "staging/stg_customers.sql",
            expected_dependencies: vec!["raw_data.customers"],
        },
        TestFixture {
            path: "staging/stg_transactions.sql",
            expected_dependencies: vec!["raw_data.transactions"],
        },
        TestFixture {
            path: "staging/stg_accounts.sql",
            expected_dependencies: vec!["raw_data.accounts"],
        },
        TestFixture {
            path: "staging/stg_merchants.sql",
            expected_dependencies: vec!["raw_data.merchants"],
        },
        TestFixture {
            path: "marts/core/merchant_summary.sql",
            expected_dependencies: vec!["staging.stg_merchants", "staging.stg_transactions"],
        },
        TestFixture {
            path: "marts/finance/daily_trends.sql",
            expected_dependencies: vec!["staging.stg_transactions", "staging.stg_merchants"],
        },
        TestFixture {
            path: "marts/finance/monthly_trends.sql",
            expected_dependencies: vec!["staging.stg_transactions", "staging.stg_merchants"],
        },
        TestFixture {
            path: "marts/finance/spending_categories.sql",
            expected_dependencies: vec!["staging.stg_transactions", "staging.stg_merchants"],
        },
        TestFixture {
            path: "marts/core/customer_summary.sql",
            expected_dependencies: vec![
                "staging.stg_customers",
                "staging.stg_accounts",
                "staging.stg_transactions",
            ],
        },
        TestFixture {
            path: "marts/finance/recurring_analysis.sql",
            expected_dependencies: vec![
                "staging.stg_transactions",
                "staging.stg_merchants",
                "staging.stg_customers",
                "staging.stg_accounts",
            ],
        },
    ]
}

/// Test with realistic SQL examples from the demo project
#[test]
fn test_real_sql_models() -> Result<(), Box<dyn std::error::Error>> {
    let fixtures = get_test_fixtures();
    let mut errors = Vec::new();

    for fixture in fixtures {
        match parse_dependencies(fixture.path) {
            Ok(deps) => {
                if let Err(err) =
                    check_dependencies(&deps, &fixture.expected_dependencies, fixture.path)
                {
                    errors.push(err);
                }
            }
            Err(err) => {
                errors.push(format!("Error parsing {}: {}", fixture.path, err));
            }
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "Dependency check failed with errors:\n{}",
            errors.join("\n")
        )
        .into());
    }

    Ok(())
}

/// Test the extraction of tables from a simple query
#[test]
fn test_simple_query() {
    let sql = "SELECT * FROM users JOIN orders ON users.id = orders.user_id";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    assert!(deps.contains(&"users".to_string()));
    assert!(deps.contains(&"orders".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 2);
}

/// Test extraction of tables from a query with schema prefixes
#[test]
fn test_schema_prefixed_tables() {
    let sql = "SELECT * FROM public.users JOIN sales.orders ON users.id = orders.user_id";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    assert!(deps.contains(&"public.users".to_string()));
    assert!(deps.contains(&"sales.orders".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 2);
}

/// Test extraction of tables from a query with CTEs
#[test]
fn test_query_with_cte() {
    let sql = "
        WITH user_orders AS (
            SELECT u.*, o.order_id 
            FROM users u
            JOIN orders o ON u.id = o.user_id
        )
        SELECT * FROM user_orders JOIN products p ON p.id = user_orders.product_id
    ";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    // We should have users, orders, products, and the CTE user_orders
    assert!(deps.contains(&"users".to_string()));
    assert!(deps.contains(&"orders".to_string()));
    assert!(deps.contains(&"products".to_string()));
    assert!(deps.contains(&"user_orders".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 4);
}

/// Test extraction of tables from a query with subqueries
#[test]
fn test_query_with_subquery() {
    let sql = "
        SELECT * 
        FROM users
        WHERE id IN (
            SELECT user_id FROM orders WHERE total > 100
        )
    ";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    assert!(deps.contains(&"users".to_string()));
    assert!(deps.contains(&"orders".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 2);
}

/// Test extraction of tables from a complex query with multiple CTEs and subqueries
#[test]
fn test_complex_query() {
    let sql = "
        WITH recent_orders AS (
            SELECT * FROM orders WHERE order_date > '2023-01-01'
        ),
        premium_users AS (
            SELECT u.* 
            FROM users u
            JOIN user_subscriptions s ON u.id = s.user_id
            WHERE s.plan = 'premium'
        )
        SELECT 
            u.name,
            o.order_id,
            (SELECT COUNT(*) FROM order_items WHERE order_id = o.order_id) AS item_count
        FROM premium_users u
        JOIN recent_orders o ON u.id = o.user_id
        WHERE u.id IN (SELECT user_id FROM user_activity WHERE last_login > '2023-06-01')
    ";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    // Check for all expected tables
    let expected = vec![
        "orders",
        "users",
        "user_subscriptions",
        "recent_orders",
        "premium_users",
        "order_items",
        "user_activity",
    ];

    for &table in &expected {
        assert!(
            deps.contains(&table.to_string()),
            "Missing expected table: {}. Found: {:?}",
            table,
            deps
        );
    }

    // For now, we're focusing on making sure all expected tables are found
    // We may get additional tables from function calls, etc.
    assert!(deps.len() >= expected.len());
}

/// Test extraction of tables from a query with derived tables (subqueries in FROM)
#[test]
fn test_derived_tables() {
    let sql = "
        SELECT a.*, b.*
        FROM (SELECT id, name FROM users WHERE active = true) a
        JOIN (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) b
        ON a.id = b.user_id
    ";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    assert!(deps.contains(&"users".to_string()));
    assert!(deps.contains(&"orders".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 2);
}

/// Test multiple nested levels of CTEs
#[test]
fn test_nested_ctes() {
    let sql = "
        WITH level1 AS (
            SELECT * FROM base_table
        ),
        level2 AS (
            WITH inner_cte AS (
                SELECT * FROM level1 JOIN other_table ON level1.id = other_table.id
            )
            SELECT * FROM inner_cte
        )
        SELECT * FROM level2
    ";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    // We should have base_table, other_table, level1, inner_cte, level2
    assert!(deps.contains(&"base_table".to_string()));
    assert!(deps.contains(&"other_table".to_string()));
    assert!(deps.contains(&"level1".to_string()));
    assert!(deps.contains(&"inner_cte".to_string()));
    assert!(deps.contains(&"level2".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 5);
}

/// Test handling of UNION, INTERSECT, EXCEPT operations
#[test]
fn test_set_operations() {
    let sql = "
        SELECT * FROM table1
        UNION
        SELECT * FROM table2
        EXCEPT
        SELECT * FROM table3
        INTERSECT
        SELECT * FROM table4
    ";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    assert!(deps.contains(&"table1".to_string()));
    assert!(deps.contains(&"table2".to_string()));
    assert!(deps.contains(&"table3".to_string()));
    assert!(deps.contains(&"table4".to_string()));
    // Allow for additional tables (like functions)
    assert!(deps.len() >= 4);
}

/// Test function calls that might reference tables
#[test]
fn test_table_functions() {
    let sql = "SELECT * FROM table_func()";
    let dialect = DuckDbDialect {};
    let ast = SqlParser::parse_sql(&dialect, sql).unwrap();
    let deps = get_table_names(&ast);

    assert!(deps.contains(&"table_func".to_string()));
}
