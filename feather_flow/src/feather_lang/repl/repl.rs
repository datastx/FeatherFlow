use std::io::{BufRead, Write};

use crate::lexer;
use crate::token::{Token, TokenType};

pub const PROMPT: &str = ">> ";

/// Starts the REPL (Read-Eval-Print Loop).
///
/// # Parameters
/// - `input`: An object implementing `BufRead` (e.g. standard input)
/// - `output`: A mutable reference to an object implementing `Write` (e.g. standard output)
pub fn start<R: BufRead, W: Write>(mut input: R, output: &mut W) {
    loop {
        // Print the prompt and flush to ensure it appears immediately.
        write!(output, "{}", PROMPT).expect("Failed to write prompt");
        output.flush().expect("Failed to flush prompt");

        // Read a line of input from the user.
        let mut line = String::new();
        let bytes_read = input.read_line(&mut line).expect("Failed to read line");

        // If zero bytes were read, we've reached EOF.
        if bytes_read == 0 {
            break;
        }

        // Create a new lexer for the given input line.
        let mut l = lexer::Lexer::new(&line);

        // Iterate through tokens until we encounter an EOF token.
        loop {
            let tok = l.next_token();
            if tok.token_type == TokenType::Eof {
                break;
            }
            // Print the token using its Debug representation.
            writeln!(output, "{:?}", tok).expect("Failed to write token");
        }
    }
}
