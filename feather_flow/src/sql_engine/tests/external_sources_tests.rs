use super::super::sql_model::{SqlModel, SqlModelCollection};
use sqlparser::dialect::DuckDbDialect;
use std::collections::HashSet;
use std::fs;
use tempfile::tempdir;

/// Helper to assert that a hashset contains expected strings
fn assert_contains(set: &HashSet<String>, expected: &[&str]) {
    for item in expected {
        assert!(
            set.contains(&item.to_string()),
            "Expected '{}' to be in the set, but it was not found.",
            item
        );
    }
}

/// Helper to assert that a hashset is exactly equal to expected strings
fn assert_set_equals(set: &HashSet<String>, expected: &[&str]) {
    let expected_set: HashSet<String> = expected.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        set, &expected_set,
        "Sets are not equal. Set: {:?}, Expected: {:?}",
        set, expected_set
    );
}

#[test]
fn test_external_sources_basic() {
    // Create a temporary directory for our test models
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    let dialect = DuckDbDialect {};

    // Create a model that references an external source
    let model_dir = project_root.join("model_dir");
    fs::create_dir(&model_dir).unwrap();

    let sql_content = "SELECT id, name FROM external_schema.external_table";
    let model_path = model_dir.join("model.sql");
    fs::write(&model_path, sql_content).unwrap();

    // Create and parse the model
    let mut model = SqlModel::from_path(&model_path, project_root, "duckdb", &dialect).unwrap();
    model.extract_dependencies().unwrap();

    // Check that the external source is correctly identified
    assert_contains(
        &model.referenced_tables,
        &["external_schema.external_table"],
    );

    // The external_sources field should be populated by the build_dependency_graph method
    // which is called on the SqlModelCollection, not on individual models
    // So we need to add this model to a collection and build the graph

    let mut collection = SqlModelCollection::new();
    collection.add_model(model);
    collection.build_dependency_graph();

    // Get the updated model from the collection
    let model_id = "model.model_dir.model";
    let updated_model = collection
        .get_model(model_id)
        .expect("Model not found in collection");

    // Now check that the external_sources field has been populated correctly
    assert_contains(
        &updated_model.external_sources,
        &["external_schema.external_table"],
    );
    assert_eq!(updated_model.external_sources.len(), 1);

    // Verify that get_external_sources() returns the correct set
    let external_sources = updated_model.get_external_sources();
    assert_contains(&external_sources, &["external_schema.external_table"]);
    assert_eq!(external_sources.len(), 1);
}

#[test]
fn test_model_references_vs_external_sources() {
    // Create a temporary directory for our test models
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    let dialect = DuckDbDialect {};

    // Create two models: one that references an external source, and another that references the first model

    // Model A references an external source
    let model_a_dir = project_root.join("model_a");
    fs::create_dir(&model_a_dir).unwrap();
    let sql_a = "SELECT id, name FROM external_schema.external_table";
    let file_a = model_a_dir.join("model_a.sql");
    fs::write(&file_a, sql_a).unwrap();

    // Model B references Model A
    let model_b_dir = project_root.join("model_b");
    fs::create_dir(&model_b_dir).unwrap();
    let sql_b = "SELECT id, name FROM public.model_a"; // Reference to model_a
    let file_b = model_b_dir.join("model_b.sql");
    fs::write(&file_b, sql_b).unwrap();

    // Create and parse the models
    let mut model_a = SqlModel::from_path(&file_a, project_root, "duckdb", &dialect).unwrap();
    let mut model_b = SqlModel::from_path(&file_b, project_root, "duckdb", &dialect).unwrap();

    model_a.extract_dependencies().unwrap();
    model_b.extract_dependencies().unwrap();

    // Set schema for model_a to match the reference in model_b
    model_a.schema = Some("public".to_string());

    // Add to collection and build dependency graph
    let mut collection = SqlModelCollection::new();
    collection.add_model(model_a);
    collection.add_model(model_b);
    collection.build_dependency_graph();

    // Get the updated models
    let model_a_id = "model.model_a.model_a";
    let model_b_id = "model.model_b.model_b";

    let updated_model_a = collection.get_model(model_a_id).expect("Model A not found");
    let updated_model_b = collection.get_model(model_b_id).expect("Model B not found");

    // Check Model A's external sources
    assert_contains(
        &updated_model_a.external_sources,
        &["external_schema.external_table"],
    );
    assert_eq!(updated_model_a.external_sources.len(), 1);

    // Check Model B's references - should show model_a as a dependency, not an external source
    assert_contains(&updated_model_b.referenced_tables, &["public.model_a"]);
    assert_eq!(
        updated_model_b.external_sources.len(),
        0,
        "Model B should not have any external sources"
    );

    // Check dependency relationship
    assert_contains(&updated_model_a.downstream_models, &[model_b_id]);
    assert_contains(&updated_model_b.upstream_models, &[model_a_id]);
}

#[test]
fn test_complex_dependency_graph_with_external_sources() {
    // Create a temporary directory for our test models
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    let dialect = DuckDbDialect {};

    // Create a more complex scenario with multiple models and external sources

    // Model A: references two external sources
    let model_a_dir = project_root.join("model_a");
    fs::create_dir(&model_a_dir).unwrap();
    let sql_a = "SELECT t1.id, t2.name FROM ext_schema1.table1 t1 JOIN ext_schema2.table2 t2 ON t1.id = t2.id";
    let file_a = model_a_dir.join("model_a.sql");
    fs::write(&file_a, sql_a).unwrap();

    // Model B: references one external source and Model A
    let model_b_dir = project_root.join("model_b");
    fs::create_dir(&model_b_dir).unwrap();
    let sql_b =
        "SELECT a.id, e.value FROM staging.model_a a JOIN ext_schema3.table3 e ON a.id = e.id";
    let file_b = model_b_dir.join("model_b.sql");
    fs::write(&file_b, sql_b).unwrap();

    // Model C: references only other models (A and B)
    let model_c_dir = project_root.join("model_c");
    fs::create_dir(&model_c_dir).unwrap();
    let sql_c = "SELECT a.id, b.value FROM staging.model_a a JOIN staging.model_b b ON a.id = b.id";
    let file_c = model_c_dir.join("model_c.sql");
    fs::write(&file_c, sql_c).unwrap();

    // Create and parse models
    let mut model_a = SqlModel::from_path(&file_a, project_root, "duckdb", &dialect).unwrap();
    let mut model_b = SqlModel::from_path(&file_b, project_root, "duckdb", &dialect).unwrap();
    let mut model_c = SqlModel::from_path(&file_c, project_root, "duckdb", &dialect).unwrap();

    // Extract dependencies
    model_a.extract_dependencies().unwrap();
    model_b.extract_dependencies().unwrap();
    model_c.extract_dependencies().unwrap();

    // Set schemas to match references
    model_a.schema = Some("staging".to_string());
    model_b.schema = Some("staging".to_string());

    // Add to collection and build dependency graph
    let mut collection = SqlModelCollection::new();
    collection.add_model(model_a);
    collection.add_model(model_b);
    collection.add_model(model_c);
    collection.build_dependency_graph();

    // Get updated models
    let model_a_id = "model.model_a.model_a";
    let model_b_id = "model.model_b.model_b";
    let model_c_id = "model.model_c.model_c";

    let updated_model_a = collection.get_model(model_a_id).expect("Model A not found");
    let updated_model_b = collection.get_model(model_b_id).expect("Model B not found");
    let updated_model_c = collection.get_model(model_c_id).expect("Model C not found");

    // Check external sources
    assert_set_equals(
        &updated_model_a.external_sources,
        &["ext_schema1.table1", "ext_schema2.table2"],
    );
    assert_set_equals(&updated_model_b.external_sources, &["ext_schema3.table3"]);
    assert_eq!(
        updated_model_c.external_sources.len(),
        0,
        "Model C should not have any external sources"
    );

    // Check dependency relationships
    assert_contains(
        &updated_model_a.downstream_models,
        &[model_b_id, model_c_id],
    );

    assert_contains(&updated_model_b.upstream_models, &[model_a_id]);
    assert_contains(&updated_model_b.downstream_models, &[model_c_id]);

    assert_contains(&updated_model_c.upstream_models, &[model_a_id, model_b_id]);
    assert_eq!(
        updated_model_c.downstream_models.len(),
        0,
        "Model C should not have any downstream models"
    );

    // Check model depths
    assert_eq!(
        updated_model_a.depth,
        Some(0),
        "Model A should have depth 0"
    );
    assert_eq!(
        updated_model_b.depth,
        Some(1),
        "Model B should have depth 1"
    );
    assert_eq!(
        updated_model_c.depth,
        Some(2),
        "Model C should have depth 2"
    );
}

#[test]
fn test_external_sources_with_real_world_structure() {
    // Create a temporary directory for our test models
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    let dialect = DuckDbDialect {};

    // Create directories that mimic a real-world project structure
    fs::create_dir_all(project_root.join("staging/stg_raw_data")).unwrap();
    fs::create_dir_all(project_root.join("marts/core/summary_model")).unwrap();
    fs::create_dir_all(project_root.join("marts/reporting/final_report")).unwrap();

    // Staging model: references external raw data
    let sql_stg = "SELECT id, name, value FROM raw_data.source_table WHERE active = true";
    let file_stg = project_root.join("staging/stg_raw_data/stg_raw_data.sql");
    fs::write(&file_stg, sql_stg).unwrap();

    // Core model: references staging model
    let sql_core = "SELECT id, SUM(value) as total_value FROM staging.stg_raw_data GROUP BY id";
    let file_core = project_root.join("marts/core/summary_model/summary_model.sql");
    fs::write(&file_core, sql_core).unwrap();

    // Reporting model: references core model and another external source
    let sql_report = "SELECT s.id, s.total_value, e.category \
                      FROM marts.core.summary_model s \
                      JOIN reference_data.categories e ON s.id = e.id";
    let file_report = project_root.join("marts/reporting/final_report/final_report.sql");
    fs::write(&file_report, sql_report).unwrap();

    // Create and parse models
    let mut stg_model = SqlModel::from_path(&file_stg, project_root, "duckdb", &dialect).unwrap();
    let mut core_model = SqlModel::from_path(&file_core, project_root, "duckdb", &dialect).unwrap();
    let mut report_model =
        SqlModel::from_path(&file_report, project_root, "duckdb", &dialect).unwrap();

    // Extract dependencies
    stg_model.extract_dependencies().unwrap();
    core_model.extract_dependencies().unwrap();
    report_model.extract_dependencies().unwrap();

    // Set schemas to match references
    stg_model.schema = Some("staging".to_string());
    core_model.schema = Some("marts.core".to_string());

    // Add to collection and build dependency graph
    let mut collection = SqlModelCollection::new();
    collection.add_model(stg_model);
    collection.add_model(core_model);
    collection.add_model(report_model);
    collection.build_dependency_graph();

    // Get updated models
    let stg_id = "model.staging.stg_raw_data.stg_raw_data";
    let core_id = "model.marts.core.summary_model.summary_model";
    let report_id = "model.marts.reporting.final_report.final_report";

    let updated_stg = collection
        .get_model(stg_id)
        .expect("Staging model not found");
    let updated_core = collection.get_model(core_id).expect("Core model not found");
    let updated_report = collection
        .get_model(report_id)
        .expect("Report model not found");

    // Check external sources
    assert_set_equals(&updated_stg.external_sources, &["raw_data.source_table"]);
    assert_eq!(
        updated_core.external_sources.len(),
        0,
        "Core model should not have any external sources"
    );
    assert_set_equals(
        &updated_report.external_sources,
        &["reference_data.categories"],
    );

    // Check dependency relationships
    assert_contains(&updated_stg.downstream_models, &[core_id]);
    assert_eq!(
        updated_stg.upstream_models.len(),
        0,
        "Staging model should not have any upstream models"
    );

    assert_contains(&updated_core.upstream_models, &[stg_id]);
    assert_contains(&updated_core.downstream_models, &[report_id]);

    assert_contains(&updated_report.upstream_models, &[core_id]);
    assert_eq!(
        updated_report.downstream_models.len(),
        0,
        "Report model should not have any downstream models"
    );

    // Check model depths
    assert_eq!(
        updated_stg.depth,
        Some(0),
        "Staging model should have depth 0"
    );
    assert_eq!(
        updated_core.depth,
        Some(1),
        "Core model should have depth 1"
    );
    assert_eq!(
        updated_report.depth,
        Some(2),
        "Report model should have depth 2"
    );
}

#[test]
fn test_get_external_sources_method() {
    // Create a temporary directory for our test models
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    let dialect = DuckDbDialect {};

    // Create a model that references multiple external sources
    let model_dir = project_root.join("model_dir");
    fs::create_dir(&model_dir).unwrap();

    let sql_content = "SELECT a.id, b.name, c.value \
                      FROM source1.table1 a \
                      JOIN source2.table2 b ON a.id = b.id \
                      JOIN source3.table3 c ON a.id = c.id";
    let model_path = model_dir.join("model.sql");
    fs::write(&model_path, sql_content).unwrap();

    // Create and parse the model
    let mut model = SqlModel::from_path(&model_path, project_root, "duckdb", &dialect).unwrap();
    model.extract_dependencies().unwrap();

    // Add to collection and build dependency graph
    let mut collection = SqlModelCollection::new();
    collection.add_model(model);
    collection.build_dependency_graph();

    // Get the updated model from the collection
    let model_id = "model.model_dir.model";
    let updated_model = collection
        .get_model(model_id)
        .expect("Model not found in collection");

    // Test the get_external_sources() method
    let external_sources = updated_model.get_external_sources();

    // Verify that all external sources are correctly identified
    assert_contains(
        &external_sources,
        &["source1.table1", "source2.table2", "source3.table3"],
    );
    assert_eq!(external_sources.len(), 3);

    // Verify that the result is a clone, not a reference to the internal field
    let mut clone_check = external_sources.clone();
    clone_check.insert("new_source.table".to_string());
    assert_eq!(
        updated_model.external_sources.len(),
        3,
        "Original external_sources set should not have been modified"
    );
}
