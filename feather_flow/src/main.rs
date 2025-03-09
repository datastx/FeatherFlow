use sqlparser::ast::{Statement, TableFactor};
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;

fn main() {
    let sql = "SELECT id, name FROM test inner join poop on test.id = poop.id WHERE active = 1 ;";

    let dialect = DuckDbDialect {};

    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    println!("{:#?}", ast);

    let table_names = get_table_names(&ast);
    println!("Tables: {:?}", table_names);
}

fn get_table_names(statements: &[Statement]) -> Vec<String> {
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

fn collect_table_names(table_factor: &TableFactor, table_names: &mut Vec<String>) {
    if let TableFactor::Table { name, .. } = table_factor {
        table_names.push(name.to_string());
    }
}
