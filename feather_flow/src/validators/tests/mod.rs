//! Integration tests for validators

pub mod directory_structure_tests {
    use crate::validators::{validate_model_structure, validate_models_directory};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_model_directory(
        parent_dir: &std::path::Path,
        model_name: &str,
        create_sql: bool,
        create_yml: bool,
        extra_files: &[&str],
    ) -> std::path::PathBuf {
        let model_dir = parent_dir.join(model_name);
        fs::create_dir_all(&model_dir).unwrap();

        if create_sql {
            let sql_file = model_dir.join(format!("{}.sql", model_name));
            File::create(&sql_file)
                .unwrap()
                .write_all(b"SELECT * FROM table")
                .unwrap();
        }

        if create_yml {
            let yml_file = model_dir.join(format!("{}.yml", model_name));
            File::create(&yml_file).unwrap().write_all(b"version: 2").unwrap();
        }

        for extra_file in extra_files {
            let file_path = model_dir.join(extra_file);
            File::create(&file_path).unwrap().write_all(b"content").unwrap();
        }

        model_dir
    }

    #[test]
    fn test_validate_models_directory_nested() {
        let temp_dir = tempdir().unwrap();
        let models_dir = temp_dir.path().join("models");
        
        // Create directory structure
        let staging_dir = models_dir.join("staging");
        let marts_dir = models_dir.join("marts");
        let finance_dir = marts_dir.join("finance");
        
        fs::create_dir_all(&staging_dir).unwrap();
        fs::create_dir_all(&finance_dir).unwrap();
        
        // Create valid model directories
        let stg_customers_dir = create_test_model_directory(&staging_dir, "stg_customers", true, true, &[]);
        let _stg_orders_dir = create_test_model_directory(&staging_dir, "stg_orders", true, true, &[]);
        
        // Create invalid model directories
        let daily_trends_dir = create_test_model_directory(&finance_dir, "daily_trends", true, false, &[]); // Missing YML
        let _monthly_trends_dir = create_test_model_directory(&finance_dir, "monthly_trends", false, true, &[]); // Missing SQL
        let spending_categories_dir = create_test_model_directory(&finance_dir, "spending_categories", true, true, &["notes.md"]); // Extra file
        
        // Run validation
        let results = validate_models_directory(&models_dir);
        
        // Verify results
        assert_eq!(results.len(), 5); // Should find all 5 model directories
        
        let valid_count = results.iter().filter(|r| r.is_valid).count();
        let invalid_count = results.iter().filter(|r| !r.is_valid).count();
        
        assert_eq!(valid_count, 2); // stg_customers and stg_orders should be valid
        assert_eq!(invalid_count, 3); // The other three should be invalid
        
        // Check specific results
        let stg_customers_result = results.iter().find(|r| r.path == stg_customers_dir).unwrap();
        assert!(stg_customers_result.is_valid);
        
        let daily_trends_result = results.iter().find(|r| r.path == daily_trends_dir).unwrap();
        assert!(!daily_trends_result.is_valid);
        assert!(daily_trends_result.errors[0].contains("Missing YAML file"));
        
        let spending_categories_result = results.iter().find(|r| r.path == spending_categories_dir).unwrap();
        assert!(!spending_categories_result.is_valid);
        assert!(spending_categories_result.errors[0].contains("Unexpected file"));
    }

    #[test]
    fn test_demo_project_structure() {
        use std::path::PathBuf;
        
        // Path to the demo project models directory
        let models_dir = PathBuf::from("/workspaces/FeatherFlow/demo_project/models");
        
        if models_dir.exists() {
            let results = validate_models_directory(&models_dir);
            
            // All model directories should be valid
            for result in &results {
                assert!(
                    result.is_valid,
                    "Invalid model structure for {}: {:?}",
                    result.path.display(),
                    result.errors
                );
            }
            
            // Check that we found all expected model directories
            let model_count = results.len();
            assert!(
                model_count > 0,
                "Expected to find model directories in {}",
                models_dir.display()
            );
            println!("Validated {} model directories in demo project", model_count);
        } else {
            println!("Demo project models directory not found: {}", models_dir.display());
        }
    }
}