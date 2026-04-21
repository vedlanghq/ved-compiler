pub mod lexer;
pub mod parser;
pub mod ast;
pub mod semantic;
pub mod codegen;

use miette::{Diagnostic, Report};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[error("Compilation Failed")]
pub struct CompileError {
    #[source_code]
    pub src: String,
    
    #[related]
    pub errors: Vec<semantic::SemanticError>,
}

#[derive(Error, Diagnostic, Debug)]
#[error("Syntax Errors Detected")]
pub struct ParseCompileError {
    #[source_code]
    pub src: String,
    
    #[related]
    pub errors: Vec<parser::ParseError>,
}

pub fn compile_source(source: &str) -> Result<codegen::BytecodeProgram, Report> {
    let tokens = lexer::lex(source);
    for (t, _span) in &tokens {
        if let lexer::Token::Unknown(c) = t {
            // Note: Single unknown char error could be a diagnostic too
            return Err(miette::miette!("Unknown character: {}", c));
        }
    }

    let ast = match parser::parse(tokens) {
        Ok(ast) => ast,
        Err(e) => return Err(ParseCompileError {
            src: source.to_string(),
            errors: vec![e],
        }.into()),
    };

    let mut validator = semantic::SemanticValidator::new();
    if let Err(errors) = validator.validate(&ast) {
        return Err(CompileError {
            src: source.to_string(),
            errors,
        }.into());
    }

    let generator = codegen::CodeGenerator::new();
    let program = generator.generate(&ast);

    Ok(program)
}
