pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantics;

pub use error::CompilerError;

/// Compiles Ved source code into an intermediate representation.
pub fn compile_source(source: &str) -> Result<(), CompilerError> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(&tokens)?;
    semantics::analyze(&ast)?;
    // compiler::generate_bytecode(&ast)?;
    Ok(())
}

