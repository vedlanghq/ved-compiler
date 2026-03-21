use crate::ast::SystemNode;
use crate::error::CompilerError;
use crate::lexer::token::Token;

pub fn parse(tokens: &[Token]) -> Result<SystemNode, CompilerError> {
    if tokens.is_empty() {
        return Err(CompilerError::ParseError("Unexpected EOF".to_string()));
    }
    
    // Stub parser. To be implemented as recursive descent taking `&[Token]` and outputting the AST.
    Ok(SystemNode {
        name: "mock_system".to_string(),
        domains: vec![],
    })
}
