#[cfg(test)]
mod tests {
    use super::*;  // Import everything from the lexer module

    #[test]
    fn test_single_character_tokens() {
        let input = "=+(){},;";  // A simple sequence of single-character tokens
        let mut lexer = Lexer::new(input);

        // Expected sequence of token types and literals
        let expected_tokens = [
            (TokenType::Assign, "="),
            (TokenType::Plus, "+"),
            (TokenType::LParen, "("),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::RBrace, "}"),
            (TokenType::Comma, ","),
            (TokenType::Semicolon, ";"),
            (TokenType::EOF, ""),  // End of input should yield EOF token
        ];

        for (expected_type, expected_literal) in expected_tokens.iter() {
            let tok = lexer.next_token();
            assert_eq!(&tok.kind, expected_type, "Token type mismatch");
            assert_eq!(tok.literal, *expected_literal, "Token literal mismatch");
        }
    }

    #[test]
    fn test_token_sequences_and_keywords() {
        let input = r#"
            let five = 5;
            let ten = 10;
            let add = fn(x, y) {
                x + y;
            };
            let result = add(five, ten);
            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;
        "#;

        let mut lexer = Lexer::new(input);

        // List of expected tokens: (TokenType, literal)
        let tests = [
            // let five = 5;
            (TokenType::Let, "let"),
            (TokenType::Ident, "five"),
            (TokenType::Assign, "="),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),

            // let ten = 10;
            (TokenType::Let, "let"),
            (TokenType::Ident, "ten"),
            (TokenType::Assign, "="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),

            // let add = fn(x, y) { ... };
            (TokenType::Let, "let"),
            (TokenType::Ident, "add"),
            (TokenType::Assign, "="),
            (TokenType::Function, "fn"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "x"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "y"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Ident, "x"),
            (TokenType::Plus, "+"),
            (TokenType::Ident, "y"),
            (TokenType::Semicolon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Semicolon, ";"),

            // let result = add(five, ten);
            (TokenType::Let, "let"),
            (TokenType::Ident, "result"),
            (TokenType::Assign, "="),
            (TokenType::Ident, "add"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "five"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "ten"),
            (TokenType::RParen, ")"),
            (TokenType::Semicolon, ";"),

            // !-/*5;
            (TokenType::Bang, "!"),
            (TokenType::Minus, "-"),
            (TokenType::Slash, "/"),
            (TokenType::Asterisk, "*"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),

            // 5 < 10 > 5;
            (TokenType::Int, "5"),
            (TokenType::LT, "<"),
            (TokenType::Int, "10"),
            (TokenType::GT, ">"),
            (TokenType::Int, "5"),
            (TokenType::Semicolon, ";"),

            // if (5 < 10) { return true; } else { return false; }
            (TokenType::If, "if"),
            (TokenType::LParen, "("),
            (TokenType::Int, "5"),
            (TokenType::LT, "<"),
            (TokenType::Int, "10"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::True, "true"),
            (TokenType::Semicolon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Else, "else"),
            (TokenType::LBrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::False, "false"),
            (TokenType::Semicolon, ";"),
            (TokenType::RBrace, "}"),

            // 10 == 10;
            (TokenType::Int, "10"),
            (TokenType::Eq, "=="),
            (TokenType::Int, "10"),
            (TokenType::Semicolon, ";"),

            // 10 != 9;
            (TokenType::Int, "10"),
            (TokenType::NotEq, "!="),
            (TokenType::Int, "9"),
            (TokenType::Semicolon, ";"),

            // End of input
            (TokenType::EOF, ""),
        ];

        for (expected_type, expected_literal) in tests.iter() {
            let tok = lexer.next_token();
            assert_eq!(&tok.kind, expected_type, "Token type mismatch for literal `{}`", tok.literal);
            assert_eq!(tok.literal, *expected_literal, "Token literal mismatch");
        }
    }

    #[test]
    fn test_identifier_vs_keyword() {
        let input = "foobar let true false if else";
        let mut lexer = Lexer::new(input);

        // foobar -> ident, let -> keyword, true/false -> keywords, if/else -> keywords
        let expected_tokens = [
            (TokenType::Ident, "foobar"),
            (TokenType::Let, "let"),
            (TokenType::True, "true"),
            (TokenType::False, "false"),
            (TokenType::If, "if"),
            (TokenType::Else, "else"),
            (TokenType::EOF, ""),
        ];

        for (expected_type, expected_literal) in expected_tokens.iter() {
            let tok = lexer.next_token();
            assert_eq!(&tok.kind, expected_type);
            assert_eq!(tok.literal, *expected_literal);
        }
    }

    #[test]
    fn test_illegal_character() {
        let input = "@";
        let mut lexer = Lexer::new(input);
        let tok = lexer.next_token();
        assert_eq!(tok.kind, TokenType::Illegal);
        assert_eq!(tok.literal, "@");
    }
}
