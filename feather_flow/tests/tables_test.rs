use feather_flow::sql_engine::tables::{ColumnDef, SqlType, TableManager, TableSchema};

fn create_test_table_manager() -> TableManager {
    let mut manager = TableManager::new();

    // Define users table
    let users_schema = TableSchema {
        name: "users".to_string(),
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                data_type: SqlType::Integer,
                nullable: false,
            },
            ColumnDef {
                name: "name".to_string(),
                data_type: SqlType::Text,
                nullable: false,
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
    };

    // Define orders table
    let orders_schema = TableSchema {
        name: "orders".to_string(),
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                data_type: SqlType::Integer,
                nullable: false,
            },
            ColumnDef {
                name: "user_id".to_string(),
                data_type: SqlType::Integer,
                nullable: false,
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
    };

    manager.register_schema(users_schema);
    manager.register_schema(orders_schema);

    manager
}

#[test]
fn test_get_table_names() {
    let manager = create_test_table_manager();

    let table_names = manager.get_table_names();
    assert_eq!(table_names.len(), 2);
    assert!(table_names.contains(&"users".to_string()));
    assert!(table_names.contains(&"orders".to_string()));
}

#[test]
fn test_get_schema() {
    let manager = create_test_table_manager();

    let users_schema = manager.get_schema("users").unwrap();
    assert_eq!(users_schema.name, "users");
    assert_eq!(users_schema.columns.len(), 2);

    let non_existent = manager.get_schema("non_existent");
    assert!(non_existent.is_none());
}

#[test]
fn test_get_column_names() {
    let manager = create_test_table_manager();

    let user_columns = manager.get_column_names("users").unwrap();
    assert_eq!(user_columns.len(), 2);
    assert!(user_columns.contains(&"id".to_string()));
    assert!(user_columns.contains(&"name".to_string()));

    assert!(manager.get_column_names("non_existent").is_none());
}
