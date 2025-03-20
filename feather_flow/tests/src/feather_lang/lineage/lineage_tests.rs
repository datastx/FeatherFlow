//! Integration tests for the SQL lineage functionality
use feather_flow::sql_engine::lineage::{extract_column_lineage, generate_lineage_graph};

#[test]
fn test_simple_select_lineage() {
    let sql = "SELECT id, name FROM users";
    
    let lineage = extract_column_lineage(sql).unwrap();
    assert_eq!(lineage.len(), 2);
    
    // Check first column lineage
    assert_eq!(lineage[0].target.column, "id");
    assert_eq!(lineage[0].sources.len(), 1);
    assert_eq!(lineage[0].sources[0].table, Some("users".to_string()));
    assert_eq!(lineage[0].sources[0].column, "id");
    assert_eq!(lineage[0].transformation, "direct");
    
    // Check second column lineage
    assert_eq!(lineage[1].target.column, "name");
    assert_eq!(lineage[1].sources.len(), 1);
    assert_eq!(lineage[1].sources[0].table, Some("users".to_string()));
    assert_eq!(lineage[1].sources[0].column, "name");
    assert_eq!(lineage[1].transformation, "direct");
}

#[test]
fn test_select_with_join_lineage() {
    let sql = "SELECT 
                 c.id as customer_id, 
                 c.name as customer_name,
                 o.id as order_id,
                 o.amount as order_amount
               FROM customers c
               JOIN orders o ON c.id = o.customer_id";
    
    let lineage = extract_column_lineage(sql).unwrap();
    assert_eq!(lineage.len(), 4);
    
    // Check customer_id lineage
    assert_eq!(lineage[0].target.column, "customer_id");
    assert_eq!(lineage[0].sources.len(), 1);
    assert_eq!(lineage[0].sources[0].table, Some("customers".to_string()));
    assert_eq!(lineage[0].sources[0].column, "id");
    
    // Check customer_name lineage
    assert_eq!(lineage[1].target.column, "customer_name");
    assert_eq!(lineage[1].sources.len(), 1);
    assert_eq!(lineage[1].sources[0].table, Some("customers".to_string()));
    assert_eq!(lineage[1].sources[0].column, "name");
    
    // Check order_id lineage
    assert_eq!(lineage[2].target.column, "order_id");
    assert_eq!(lineage[2].sources.len(), 1);
    assert_eq!(lineage[2].sources[0].table, Some("orders".to_string()));
    assert_eq!(lineage[2].sources[0].column, "id");
    
    // Check order_amount lineage
    assert_eq!(lineage[3].target.column, "order_amount");
    assert_eq!(lineage[3].sources.len(), 1);
    assert_eq!(lineage[3].sources[0].table, Some("orders".to_string()));
    assert_eq!(lineage[3].sources[0].column, "amount");
}

#[test]
fn test_select_with_expressions_lineage() {
    let sql = "SELECT 
                 id,
                 price,
                 quantity,
                 price * quantity as total,
                 CASE 
                   WHEN price * quantity > 100 THEN 'High Value'
                   ELSE 'Standard'
                 END as order_tier
               FROM orders";
    
    let lineage = extract_column_lineage(sql).unwrap();
    assert_eq!(lineage.len(), 5);
    
    // Check total column lineage (expression)
    assert_eq!(lineage[3].target.column, "total");
    assert_eq!(lineage[3].transformation, "expression");
    assert_eq!(lineage[3].sources.len(), 2);
    
    // Verify source columns
    let source_columns: Vec<String> = lineage[3].sources.iter()
        .map(|s| s.column.clone())
        .collect();
    assert!(source_columns.contains(&"price".to_string()));
    assert!(source_columns.contains(&"quantity".to_string()));
    
    // Check order_tier column lineage (case expression)
    assert_eq!(lineage[4].target.column, "order_tier");
    assert_eq!(lineage[4].transformation, "case_when");
    assert_eq!(lineage[4].sources.len(), 2);
    
    // Same source columns as total
    let source_columns2: Vec<String> = lineage[4].sources.iter()
        .map(|s| s.column.clone())
        .collect();
    assert!(source_columns2.contains(&"price".to_string()));
    assert!(source_columns2.contains(&"quantity".to_string()));
}

#[test]
fn test_select_with_aggregations_lineage() {
    let sql = "SELECT 
                 customer_id,
                 COUNT(*) as order_count,
                 SUM(amount) as total_spent,
                 AVG(amount) as avg_order_value,
                 MAX(order_date) as latest_order
               FROM orders
               GROUP BY customer_id";
    
    let lineage = extract_column_lineage(sql).unwrap();
    assert_eq!(lineage.len(), 5);
    
    // Check aggregation columns
    assert_eq!(lineage[1].target.column, "order_count");
    assert_eq!(lineage[1].transformation, "aggregation");
    
    assert_eq!(lineage[2].target.column, "total_spent");
    assert_eq!(lineage[2].transformation, "aggregation");
    assert_eq!(lineage[2].sources[0].column, "amount");
    
    assert_eq!(lineage[3].target.column, "avg_order_value");
    assert_eq!(lineage[3].transformation, "aggregation");
    assert_eq!(lineage[3].sources[0].column, "amount");
    
    assert_eq!(lineage[4].target.column, "latest_order");
    assert_eq!(lineage[4].transformation, "aggregation");
    assert_eq!(lineage[4].sources[0].column, "order_date");
}

#[test]
fn test_cte_lineage() {
    let sql = "WITH order_summary AS (
                 SELECT 
                   customer_id,
                   COUNT(*) as order_count,
                   SUM(amount) as total_spent
                 FROM orders
                 GROUP BY customer_id
               )
               SELECT 
                 c.id,
                 c.name,
                 COALESCE(os.order_count, 0) as order_count,
                 COALESCE(os.total_spent, 0) as total_spent
               FROM customers c
               LEFT JOIN order_summary os ON c.id = os.customer_id";
    
    // Basic check that it doesn't fail - CTEs need more complex handling
    let lineage = extract_column_lineage(sql);
    assert!(lineage.is_ok());
}

#[test]
fn test_graph_generation() {
    let sql = "SELECT c.id, c.name, o.order_date 
               FROM customers c 
               JOIN orders o ON c.id = o.customer_id";
    
    let lineage = extract_column_lineage(sql).unwrap();
    let graph = generate_lineage_graph(&lineage);
    
    // Basic checks on the graph output
    assert!(graph.starts_with("digraph lineage {"));
    assert!(graph.contains("rankdir=LR"));
    assert!(graph.contains("fillcolor=lightblue"));
    assert!(graph.ends_with("}\n"));
}