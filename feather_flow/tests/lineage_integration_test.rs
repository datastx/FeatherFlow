//! Integration tests for column lineage tracking

use feather_flow::sql_engine::lineage::{extract_column_lineage, generate_lineage_graph};

/// Test lineage extraction from a simple model
#[test]
fn test_model_file_lineage() {
    let sql = r#"SELECT 
    c.id AS customer_id,
    c.name AS customer_name,
    c.email AS customer_email,
    COUNT(o.order_id) AS order_count,
    SUM(o.amount) AS total_amount
FROM staging.stg_customers c
LEFT JOIN staging.stg_orders o ON c.id = o.customer_id
GROUP BY c.id, c.name, c.email"#;
    
    let lineage = extract_column_lineage(sql).unwrap();
    
    // Should have 5 columns in the result
    assert_eq!(lineage.len(), 5);
    
    // Check a few of the columns
    let customer_id = lineage.iter().find(|l| l.target.column == "customer_id");
    assert!(customer_id.is_some(), "Failed to find customer_id column in lineage");
    
    // Check a couple of aggregations
    let order_count = lineage.iter().find(|l| l.target.column == "order_count");
    assert!(order_count.is_some(), "Failed to find order_count column in lineage");
    if let Some(order_count) = order_count {
        assert_eq!(order_count.transformation, "aggregation");
    }
    
    let total_amount = lineage.iter().find(|l| l.target.column == "total_amount");
    assert!(total_amount.is_some(), "Failed to find total_amount column in lineage");
    if let Some(total_amount) = total_amount {
        assert_eq!(total_amount.transformation, "aggregation");
    }
    
    // Generate a graph visualization and check the basic format
    let graph = generate_lineage_graph(&lineage);
    assert!(graph.starts_with("digraph lineage {"));
    assert!(graph.contains("rankdir=LR"));
}

/// Test lineage extraction from a staging model
#[test]
fn test_staging_model_lineage() {
    let sql = r#"SELECT 
    id,
    name,
    email,
    created_at
FROM raw_data.customers"#;
    
    let lineage = extract_column_lineage(sql).unwrap();
    
    // Should have 4 columns in the result
    assert_eq!(lineage.len(), 4);
    
    // All transformations should be direct
    for item in &lineage {
        assert_eq!(item.transformation, "direct");
        assert!(item.sources[0].table.is_some());
    }
}