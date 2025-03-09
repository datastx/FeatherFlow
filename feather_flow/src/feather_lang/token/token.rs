use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TokenType {
    Illegal,
    Eof,
    Ident,
    Int,
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Eq,
    NotEq,
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    Rbrace,
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

// Lazy static initialization of the keywords map
static KEYWORDS: Lazy<HashMap<&'static str, TokenType>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("fn", TokenType::Function);
    m.insert("let", TokenType::Let);
    m.insert("true", TokenType::True);
    m.insert("false", TokenType::False);
    m.insert("if", TokenType::If);
    m.insert("else", TokenType::Else);
    m.insert("return", TokenType::Return);
    m
});

pub fn lookup_ident(ident: &str) -> TokenType {
    if let Some(&token_type) = KEYWORDS.get(ident) {
        token_type
    } else {
        TokenType::Ident
    }
}
