#[derive(Debug, Clone)]
pub enum CompilerError {
    LexerError(String),
    ParseError(String),
    SemanticError(String),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            CompilerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CompilerError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}
