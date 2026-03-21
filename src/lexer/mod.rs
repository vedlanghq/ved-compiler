pub mod token;
use crate::error::CompilerError;
use token::{Token, TokenKind};

pub fn tokenize(_source: &str) -> Result<Vec<Token>, CompilerError> {
    let mut tokens = Vec::new();
    // This is a stub lexer. A full recursive descent tokenization will go here.
    tokens.push(Token::new(TokenKind::Eof, Default::default(), Default::default()));
    Ok(tokens)
}
