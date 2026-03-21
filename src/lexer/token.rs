#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Domain,
    State,
    Transition,
    Goal,
    Step,
    Emit,
    Trigger,
    Send,
    System,
    Slice,

    // Identifiers and Literals
    Identifier(String),
    StringLiteral(String),
    IntLiteral(i64),
    BoolLiteral(bool),

    // Symbols
    LeftBrace,    // {
    RightBrace,   // }
    LeftParen,    // (
    RightParen,   // )
    Colon,        // :
    Equal,        // =
    Equals,       // ==
    Plus,         // +
    Minus,        // -
    
    // Core
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}
