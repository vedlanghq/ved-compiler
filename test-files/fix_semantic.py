import re

with open('src/semantic.rs', 'r') as f:
    t = f.read()

t = t.replace('use crate::ast::{Ast, Statement, StatementKind, Expr, ExprKind};',
'''use crate::ast::{Ast, Statement, StatementKind, Expr, ExprKind};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use crate::lexer::Span;''')

t = t.replace('''#[derive(Debug)]
pub struct SemanticError {
    pub message: String,
}''', '''#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code(ved::semantic))]
pub struct SemanticError {
    pub message: String,
    #[label("here")]
    pub span: SourceSpan,
}

fn to_span(span: Span) -> SourceSpan {
    (span.offset, span.len).into()
}''')

t = t.replace('message: format!("Domain \'{}\': Unknown type \'{}\' for field \'{}\'", domain.name, t, field.name),',
              'message: format!("Domain \'{}\': Unknown type \'{}\' for field \'{}\'", domain.name, t, field.name),\n                                span: to_span(field.span),')

t = t.replace('message: format!("Domain \'{}\': Duplicate state field \'{}\'", domain.name, field.name),',
              'message: format!("Domain \'{}\': Duplicate state field \'{}\'", domain.name, field.name),\n                                span: to_span(field.span),')

t = t.replace('message: format!("E002: Unknown service \'{}\' in deployment statement.", deploy.service),',
              'message: format!("E002: Unknown service \'{}\' in deployment statement.", deploy.service),\n                        span: to_span(stmt.span),')

t = t.replace('message: format!(\n                            "E001: Execution authority violation\\nManual context mutation detected.\\n\\nRequired: environment-bound capability\\nFound: undeclared environment \'{}\'\\n\\nResolution:\\nBind operation to a declared environment block.", deploy.target_environment\n                        ),',
              'message: format!(\n                            "E001: Execution authority violation\\nManual context mutation detected.\\n\\nRequired: environment-bound capability\\nFound: undeclared environment \'{}\'\\n\\nResolution:\\nBind operation to a declared environment block.", deploy.target_environment\n                        ),\n                        span: to_span(stmt.span),')


t = t.replace('message: format!("Domain \'{}\': Reference to undefined state variable \'{}\'", domain_name, name),',
              'message: format!("Domain \'{}\': Reference to undefined state variable \'{}\'", domain_name, name),\n                        span: to_span(expr.span),')

t = t.replace('message: format!("Domain \'{}\': Illegal mutation of \'{}\'. Goals must be strictly read-only and side-effect free.", domain_name, target),',
              'message: format!("Domain \'{}\': Illegal mutation of \'{}\'. Goals must be strictly read-only and side-effect free.", domain_name, target),\n                        span: to_span(expr.span),')

t = t.replace('message: format!("Domain \'{}\': Cannot assign to undefined state variable \'{}\'", domain_name, target),',
              'message: format!("Domain \'{}\': Cannot assign to undefined state variable \'{}\'", domain_name, target),\n                        span: to_span(expr.span),')

t = t.replace('message: format!("Domain \'{}\': Unbounded `while(true)` loops are forbidden to preserve slice computation bounds.", domain_name),',
              'message: format!("Domain \'{}\': Unbounded `while(true)` loops are forbidden to preserve slice computation bounds.", domain_name),\n                        span: to_span(expr.span),')

t = t.replace('message: format!("Domain \'{}\': Illegal effect \'send\'. Goals must be strictly side-effect free.", domain_name),',
              'message: format!("Domain \'{}\': Illegal effect \'send\'. Goals must be strictly side-effect free.", domain_name),\n                        span: to_span(expr.span),')

t = t.replace('message: format!("Domain \'{}\': Illegal effect \'send_high\'. Goals must be strictly side-effect free.", domain_name),',
              'message: format!("Domain \'{}\': Illegal effect \'send_high\'. Goals must be strictly side-effect free.", domain_name),\n                        span: to_span(expr.span),')

t = t.replace('message: format!(\n                            "E001: Execution authority violation\\nManual context mutation detected: `shell(...)` is forbidden.\\n\\nRequired: environment-bound capability\\nFound: local session context\\n\\nResolution:\\nBind operation to declared environment block."\n                        ),',
              'message: format!(\n                            "E001: Execution authority violation\\nManual context mutation detected: `shell(...)` is forbidden.\\n\\nRequired: environment-bound capability\\nFound: local session context\\n\\nResolution:\\nBind operation to declared environment block."\n                        ),\n                        span: to_span(expr.span),')

with open('src/semantic.rs', 'w') as f:
    f.write(t)
