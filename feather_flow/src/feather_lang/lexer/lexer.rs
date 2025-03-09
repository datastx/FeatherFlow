/// TokenType represents the different kinds of tokens in our language.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    // Special token types
    Illegal,
    EOF,

    // Identifiers + literals
    Ident,  // Identifier (e.g., variable names, function names)
    Int,    // Integer literal

    // Operators
    Assign, // '='
    Plus,   // '+'
    Minus,  // '-'
    Bang,   // '!'
    Asterisk, // '*'
    Slash,    // '/'

    LT,  // '<'
    GT,  // '>'
    Eq,     // '=='
    NotEq,  // '!='

    // Delimiters
    Comma,     
    Semicolon, 

    LParen, // '('
    RParen, // ')'
    LBrace, // '{'
    RBrace, // '}'

    // Keywords
    Function, // "fn"
    Let,      // "let"
    True,     // "true"
    False,    // "false"
    If,       // "if"
    Else,     // "else"
    Return,   // "return"
}

/// Token holds a token type and the literal text that it represents.
#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenType,
    pub literal: &'a str,
}

/// Lexer struct that iterates over an input string and produces tokens.
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,      // Current position in input (byte index of current char)
    read_position: usize, // Next position to read (byte index of next char)
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    /// Create a new Lexer from an input string.
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            current_char: None,
        };
        lexer.read_char(); // Initialize the first character
        lexer
    }

    /// Read the next character from input and advance the position in the input.
    /// Sets current_char to None when end of input is reached.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            // End of input reached
            self.current_char = None;
        } else {
            // Get the next byte and convert to char (ASCII assumed)
            let next_byte = self.input.as_bytes()[self.read_position];
            self.current_char = Some(next_byte as char);
        }
        // Move the position forward
        self.position = self.read_position;
        self.read_position += 1;
    }

    /// Peek at the next character without moving the lexer forward.
    /// Returns None if at end of input.
    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            // Safe to index because read_position < len
            let next_byte = self.input.as_bytes()[self.read_position];
            Some(next_byte as char)
        }
    }

    /// Skip over any whitespace characters (spaces, tabs, newlines, etc.).
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    /// Read a sequence of letters (and digits) to form an identifier or keyword.
    /// Assumes current_char is at the start of an identifier.
    fn read_identifier(&mut self) -> &'a str {
        let start_pos = self.position;
        // Continue while current_char is alphabetic or underscore or digit (for subsequent chars)
        while let Some(ch) = self.current_char {
            if is_letter(ch) || ch.is_ascii_digit() {
                // Accept letters, digits, and underscores as part of identifier
                self.read_char();
            } else {
                break;
            }
        }
        // Slice from start_pos to current position (exclusive of current position)
        &self.input[start_pos..self.position]
    }

    /// Read a sequence of digits to form a number literal.
    /// Assumes current_char is at the start of a number.
    fn read_number(&mut self) -> &'a str {
        let start_pos = self.position;
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.read_char();
            } else {
                break;
            }
        }
        &self.input[start_pos..self.position]
    }

    /// Determine the token type for an identifier (check if it's a keyword).
    fn lookup_ident(&self, ident: &str) -> TokenType {
        match ident {
            "fn"     => TokenType::Function,
            "let"    => TokenType::Let,
            "true"   => TokenType::True,
            "false"  => TokenType::False,
            "if"     => TokenType::If,
            "else"   => TokenType::Else,
            "return" => TokenType::Return,
            _        => TokenType::Ident,
        }
    }

    /// Fetch the next token from the input.
    pub fn next_token(&mut self) -> Token<'a> {
        // Skip any whitespace and position current_char at the next non-space character (or EOF)
        self.skip_whitespace();

        // Determine the token based on current_char
        let token = match self.current_char {
            // End of file/input
            None => Token { kind: TokenType::EOF, literal: "" },

            // Two-character operators
            Some('=') => {
                if self.peek_char() == Some('=') {
                    // "==" operator
                    let start = self.position;
                    self.read_char(); // consume the second '='
                    // Slice from start of "==" to the current position (which is at second '=')
                    let literal = &self.input[start..=self.position];
                    Token { kind: TokenType::Eq, literal }
                } else {
                    Token { kind: TokenType::Assign, literal: &self.input[self.position..=self.position] }
                }
            }
            Some('!') => {
                if self.peek_char() == Some('=') {
                    // "!=" operator
                    let start = self.position;
                    self.read_char();
                    let literal = &self.input[start..=self.position];
                    Token { kind: TokenType::NotEq, literal }
                } else {
                    Token { kind: TokenType::Bang, literal: &self.input[self.position..=self.position] }
                }
            }

            // Single-character tokens (operators & delimiters)
            Some('+') => Token { kind: TokenType::Plus, literal: &self.input[self.position..=self.position] },
            Some('-') => Token { kind: TokenType::Minus, literal: &self.input[self.position..=self.position] },
            Some('*') => Token { kind: TokenType::Asterisk, literal: &self.input[self.position..=self.position] },
            Some('/') => Token { kind: TokenType::Slash, literal: &self.input[self.position..=self.position] },
            Some('<') => Token { kind: TokenType::LT, literal: &self.input[self.position..=self.position] },
            Some('>') => Token { kind: TokenType::GT, literal: &self.input[self.position..=self.position] },

            Some(',') => Token { kind: TokenType::Comma, literal: &self.input[self.position..=self.position] },
            Some(';') => Token { kind: TokenType::Semicolon, literal: &self.input[self.position..=self.position] },

            Some('(') => Token { kind: TokenType::LParen, literal: &self.input[self.position..=self.position] },
            Some(')') => Token { kind: TokenType::RParen, literal: &self.input[self.position..=self.position] },
            Some('{') => Token { kind: TokenType::LBrace, literal: &self.input[self.position..=self.position] },
            Some('}') => Token { kind: TokenType::RBrace, literal: &self.input[self.position..=self.position] },

            // Identifiers and keywords
            Some(ch) if is_letter(ch) => {
                let literal = self.read_identifier();
                let kind = self.lookup_ident(literal);
                // Note: read_identifier() has already advanced current_char past the identifier
                return Token { kind, literal };
            }

            // Numbers (integer literals)
            Some(ch) if ch.is_ascii_digit() => {
                let literal = self.read_number();
                // We do not convert to an actual number here; just store the string of digits
                return Token { kind: TokenType::Int, literal };
            }

            // Any other character (not recognized)
            Some(_) => {
                // Current char is not a valid token start
                Token { kind: TokenType::Illegal, literal: &self.input[self.position..=self.position] }
            }
        };

        // Advance to the next character for subsequent calls, since we consumed this token's char(s)
        self.read_char();
        token
    }
}

/// Helper function to identify valid identifier start/part characters (ASCII letters or underscore).
fn is_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

// Implement Iterator for Lexer to allow easy iteration over tokens (excluding the final EOF if desired).
impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.next_token();
        if tok.kind == TokenType::EOF {
            None  // Stop iteration at EOF
        } else {
            Some(tok)
        }
    }
}
