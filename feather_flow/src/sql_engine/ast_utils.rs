//! Utility functions for working with SQL Abstract Syntax Trees (ASTs)
use sqlparser::ast::{Ident, Query, Statement, TableFactor};
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;

#[allow(dead_code)]
pub fn swap_sql_tables(sql: &str) -> String {
    let dialect = DuckDbDialect {};

    let mut ast = Parser::parse_sql(&dialect, sql).unwrap();

    println!("Original AST: {:#?}", ast);

    let table_names = get_table_names(&ast);
    println!("Original Tables: {:?}", table_names);

    // Modify the AST to change schema references
    modify_table_schemas(&mut ast, "private");

    println!("Modified AST: {:#?}", ast);

    // Convert the modified AST back to SQL
    let modified_sql = ast_to_sql(&ast);
    println!("Modified SQL: {}", modified_sql);
    modified_sql
}

#[allow(dead_code)]
pub fn get_table_names(statements: &[Statement]) -> Vec<String> {
    let mut table_names = Vec::new();

    for statement in statements {
        if let Statement::Query(query) = statement {
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

#[allow(dead_code)]
fn collect_table_names(table_factor: &TableFactor, table_names: &mut Vec<String>) {
    if let TableFactor::Table { name, .. } = table_factor {
        table_names.push(name.to_string());
    }
}

#[allow(dead_code)]
fn modify_table_schemas(statements: &mut [Statement], target_schema: &str) {
    for statement in statements {
        if let Statement::Query(query) = statement {
            modify_query_table_schemas(&mut *query, target_schema);
        }
    }
}

#[allow(dead_code)]
fn modify_query_table_schemas(query: &mut Query, target_schema: &str) {
    if let sqlparser::ast::SetExpr::Select(select) = &mut *query.body {
        for table_with_joins in &mut select.from {
            modify_table_schema(&mut table_with_joins.relation, target_schema);
            for join in &mut table_with_joins.joins {
                modify_table_schema(&mut join.relation, target_schema);
            }
        }
    }
}

#[allow(dead_code)]
fn modify_table_schema(table_factor: &mut TableFactor, target_schema: &str) {
    if let TableFactor::Table { name, .. } = table_factor {
        // If it's a simple table name without schema, add the target schema
        match name.0.len() {
            1 => {
                let table_name = name.0[0].value.clone();
                name.0.clear();
                name.0.push(Ident::new(target_schema));
                name.0.push(Ident::new(&table_name));
            }
            len if len > 1 => {
                name.0[0] = Ident::new(target_schema);
            }
            _ => {}
        }
    }
}

#[allow(dead_code)]
fn ast_to_sql(statements: &[Statement]) -> String {
    let mut result = String::new();

    for (i, statement) in statements.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }

        match statement {
            Statement::Query(query) => {
                result.push_str(&query_to_sql(query));
            }
            // Add other statement types as needed
            _ => result.push_str("/* Unsupported statement type */"),
        }

        result.push(';');
    }

    result
}

#[allow(dead_code)]
fn query_to_sql(query: &Query) -> String {
    match &*query.body {
        sqlparser::ast::SetExpr::Select(select) => {
            // Build SELECT clause
            let mut sql = String::from("SELECT ");

            // Project expressions (columns)
            for (i, projection) in select.projection.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }

                match projection {
                    sqlparser::ast::SelectItem::Wildcard(_) => {
                        sql.push('*');
                    }
                    sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                        match expr {
                            sqlparser::ast::Expr::Identifier(ident) => {
                                sql.push_str(&ident.value);
                            }
                            sqlparser::ast::Expr::CompoundIdentifier(idents) => {
                                sql.push_str(
                                    &idents
                                        .iter()
                                        .map(|ident| ident.value.clone())
                                        .collect::<Vec<_>>()
                                        .join("."),
                                );
                            }
                            // Handle other expression types as needed
                            _ => sql.push_str(&expr_to_sql(expr)),
                        }
                    }
                    sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } => {
                        // Handle expressions with aliases (AS)
                        match expr {
                            sqlparser::ast::Expr::Identifier(ident) => {
                                sql.push_str(&format!("{} AS {}", ident.value, alias.value));
                            }
                            sqlparser::ast::Expr::CompoundIdentifier(idents) => {
                                let column_name = idents
                                    .iter()
                                    .map(|ident| ident.value.clone())
                                    .collect::<Vec<_>>()
                                    .join(".");
                                sql.push_str(&format!("{} AS {}", column_name, alias.value));
                            }
                            _ => {
                                sql.push_str(&format!("{} AS {}", expr_to_sql(expr), alias.value));
                            }
                        }
                    }
                    // Handle other projection types as needed
                    _ => sql.push_str("/* complex projection */"),
                }
            }

            // FROM clause
            if !select.from.is_empty() {
                sql.push_str(" FROM ");

                for (i, table_with_joins) in select.from.iter().enumerate() {
                    if i > 0 {
                        sql.push_str(", ");
                    }

                    // Main table
                    sql.push_str(&table_factor_to_sql(&table_with_joins.relation));

                    // JOINs
                    for join in &table_with_joins.joins {
                        match join.join_operator {
                            sqlparser::ast::JoinOperator::Inner(_) => {
                                sql.push_str(" INNER JOIN ");
                            }
                            sqlparser::ast::JoinOperator::LeftOuter(_) => {
                                sql.push_str(" LEFT JOIN ");
                            }
                            sqlparser::ast::JoinOperator::RightOuter(_) => {
                                sql.push_str(" RIGHT JOIN ");
                            }
                            sqlparser::ast::JoinOperator::FullOuter(_) => {
                                sql.push_str(" FULL JOIN ");
                            }
                            // Add other join types as needed
                            _ => {
                                println!("Unsupported join operator: {:?}", join.join_operator);
                                sql.push_str(" JOIN ");
                            }
                        }

                        sql.push_str(&table_factor_to_sql(&join.relation));

                        // JOIN condition
                        match &join.join_operator {
                            sqlparser::ast::JoinOperator::Inner(
                                sqlparser::ast::JoinConstraint::On(expr),
                            )
                            | sqlparser::ast::JoinOperator::LeftOuter(
                                sqlparser::ast::JoinConstraint::On(expr),
                            )
                            | sqlparser::ast::JoinOperator::RightOuter(
                                sqlparser::ast::JoinConstraint::On(expr),
                            )
                            | sqlparser::ast::JoinOperator::FullOuter(
                                sqlparser::ast::JoinConstraint::On(expr),
                            ) => {
                                sql.push_str(" ON ");
                                sql.push_str(&expr_to_sql(expr));
                            }
                            _ => {}
                        }
                    }
                }
            }

            // WHERE clause
            if let Some(selection) = &select.selection {
                sql.push_str(" WHERE ");
                sql.push_str(&expr_to_sql(selection));
            }

            sql
        }
        // Handle other query types as needed
        _ => String::from("/* Unsupported query type */"),
    }
}

#[allow(dead_code)]
fn table_factor_to_sql(table_factor: &TableFactor) -> String {
    match table_factor {
        TableFactor::Table { name, alias, .. } => {
            let table_name = name
                .0
                .iter()
                .map(|ident| ident.value.clone())
                .collect::<Vec<_>>()
                .join(".");

            // Add table alias if present
            if let Some(table_alias) = alias {
                format!("{} {}", table_name, table_alias.name.value)
            } else {
                table_name
            }
        }
        // Handle other table factor types as needed
        _ => String::from("/* Unsupported table factor */"),
    }
}

#[allow(dead_code)]
fn expr_to_sql(expr: &sqlparser::ast::Expr) -> String {
    match expr {
        sqlparser::ast::Expr::BinaryOp { left, op, right } => {
            format!(
                "{} {} {}",
                expr_to_sql(left),
                match op {
                    sqlparser::ast::BinaryOperator::Eq => "=",
                    sqlparser::ast::BinaryOperator::Gt => ">",
                    sqlparser::ast::BinaryOperator::Lt => "<",
                    sqlparser::ast::BinaryOperator::GtEq => ">=",
                    sqlparser::ast::BinaryOperator::LtEq => "<=",
                    sqlparser::ast::BinaryOperator::NotEq => "<>",
                    sqlparser::ast::BinaryOperator::And => "AND",
                    sqlparser::ast::BinaryOperator::Or => "OR",
                    // Handle other operators as needed
                    _ => {
                        println!("Unsupported binary operator: {:?}", op);
                        "??"
                    }
                },
                expr_to_sql(right)
            )
        }
        sqlparser::ast::Expr::Identifier(ident) => ident.value.clone(),
        sqlparser::ast::Expr::CompoundIdentifier(idents) => idents
            .iter()
            .map(|ident| ident.value.clone())
            .collect::<Vec<_>>()
            .join("."),
        sqlparser::ast::Expr::Value(value) => {
            match value {
                sqlparser::ast::Value::Number(num, _) => num.clone(),
                sqlparser::ast::Value::SingleQuotedString(s) => format!("'{}'", s),
                sqlparser::ast::Value::Boolean(b) => b.to_string(),
                sqlparser::ast::Value::Null => String::from("NULL"),
                // Handle other value types as needed
                _ => String::from("/* unknown value */"),
            }
        }
        // Handle other expression types as needed
        _ => {
            println!("Unsupported expression type: {:?}", expr);
            String::from("/* complex expression */")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // This is correct, but unused because we're calling functions directly

    #[test]
    fn test_simple_select() {
        let input = "SELECT * FROM test";
        let expected = "SELECT * FROM private.test;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_select_with_columns() {
        let input = "SELECT id, name FROM users";
        let expected = "SELECT id, name FROM private.users;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_tables() {
        let input = "SELECT * FROM table1, table2";
        let expected = "SELECT * FROM private.table1, private.table2;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_join() {
        let input = "SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id";
        let expected =
            "SELECT * FROM private.users INNER JOIN private.orders ON users.id = orders.user_id;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_where_clause() {
        let input = "SELECT * FROM products WHERE price > 100";
        let expected = "SELECT * FROM private.products WHERE price > 100;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_existing_schema() {
        let input = "SELECT * FROM public.users";
        let expected = "SELECT * FROM private.users;";
        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_table_alias() {
        let input = "SELECT u.id, u.name FROM users u WHERE u.active = 1";
        // With the updated implementation we now correctly preserve table aliases
        let expected = "SELECT u.id, u.name FROM private.users u WHERE u.active = 1;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_complex_where() {
        let input = "SELECT * FROM products WHERE price > 100 AND category = 'electronics'";
        // The implementation now supports AND operators in WHERE clauses
        let expected =
            "SELECT * FROM private.products WHERE price > 100 AND category = 'electronics';";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_string_literal() {
        let input = "SELECT * FROM users WHERE name = 'John'";
        let expected = "SELECT * FROM private.users WHERE name = 'John';";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_column_aliases() {
        let input = "SELECT id, name AS user_name FROM users";
        // We now properly support column aliases
        let expected = "SELECT id, name AS user_name FROM private.users;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_left_join() {
        let input = "SELECT c.id, c.name, o.order_date FROM customers c LEFT JOIN orders o ON c.id = o.customer_id";
        let expected = "SELECT c.id, c.name, o.order_date FROM private.customers c LEFT JOIN private.orders o ON c.id = o.customer_id;";

        let result = swap_sql_tables(input);
        assert_eq!(result, expected);
    }
}
