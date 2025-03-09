use feather_flow::sql_engine::ast_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let input = "SELECT * FROM test";
        let expected = "SELECT * FROM private.test;";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_select_with_columns() {
        let input = "SELECT id, name FROM users";
        let expected = "SELECT id, name FROM private.users;";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_tables() {
        let input = "SELECT * FROM table1, table2";
        let expected = "SELECT * FROM private.table1, private.table2;";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_join() {
        let input = "SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id";
        let expected =
            "SELECT * FROM private.users INNER JOIN private.orders ON users.id = orders.user_id;";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_where_clause() {
        let input = "SELECT * FROM products WHERE price > 100";
        let expected = "SELECT * FROM private.products WHERE price > 100;";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_existing_schema() {
        let input = "SELECT * FROM public.users";
        let expected = "SELECT * FROM private.users;";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complex_query() {
        let input = "SELECT u.name, o.amount FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE o.status = 'completed'";
        let expected = "SELECT u.name, o.amount FROM private.users u INNER JOIN private.orders o ON u.id = o.user_id WHERE o.status = 'completed';";

        let result = ast_utils::swap_sql_tables(input);
        assert_eq!(result, expected);
    }
}
