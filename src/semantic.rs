use std::collections::HashMap;
use crate::ast::{Ast, StatementKind, Expr, ExprKind, AuthorityScope};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum VedType {
    Int,
    String,
    Bool,
    Unknown(String),
}

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code(ved::semantic))]
pub struct SemanticError {
    pub message: String,
    #[label("here")]
    pub span: SourceSpan,
}

fn to_span(span: Span) -> SourceSpan {
    (span.offset, span.len).into()
}

pub struct SemanticValidator {
    domains: HashMap<String, DomainInfo>,
    environment_capabilities: HashMap<String, Vec<String>>,
    environment_scopes: HashMap<String, String>,
}

struct DomainInfo {
    state_fields: HashMap<String, VedType>,
    required_capabilities: Vec<String>,
    scope: Option<AuthorityScope>,
}

impl SemanticValidator {
    pub fn new() -> Self {
        SemanticValidator {
            domains: HashMap::new(),
            environment_capabilities: HashMap::new(),
            environment_scopes: HashMap::new(),
        }
    }

    fn scan_effects_in_expr(&self, expr: &Expr, effects_found: &mut Vec<(String, Span)>) {
        match &expr.kind {
            ExprKind::Send { .. } => {
                effects_found.push(("send".to_string(), expr.span));
            }
            ExprKind::SendHigh { .. } => {
                effects_found.push(("send_high".to_string(), expr.span));
            }
            ExprKind::Assignment { value, .. } => {
                self.scan_effects_in_expr(value, effects_found);
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.scan_effects_in_expr(left, effects_found);
                self.scan_effects_in_expr(right, effects_found);
            }
            ExprKind::If { condition, consequence } => {
                self.scan_effects_in_expr(condition, effects_found);
                for step in consequence {
                    self.scan_effects_in_expr(step, effects_found);
                }
            }
            ExprKind::While { condition, body } => {
                self.scan_effects_in_expr(condition, effects_found);
                for step in body {
                    self.scan_effects_in_expr(step, effects_found);
                }
            }
            ExprKind::Call { arguments, .. } => {
                for arg in arguments {
                    self.scan_effects_in_expr(arg, effects_found);
                }
            }
            _ => {}
        }
    }

    pub fn validate(&mut self, ast: &Ast) -> Result<(), Vec<SemanticError>> {   
        let mut errors = Vec::new();
        let mut declared_environments = Vec::new();

        // Pass 1: Catalog all domain states and environments
        for stmt in &ast.statements {
            match &stmt.kind {
                StatementKind::DomainDecl(domain) => {
                    let mut state_fields = HashMap::new();
                    for field in &domain.state {
                        let v_type = match field.typ.as_str() {
                            "int" => VedType::Int,
                            "string" => VedType::String,
                            "bool" => VedType::Bool,
                            other => VedType::Unknown(other.to_string()),        
                        };

                        if let VedType::Unknown(ref t) = v_type {
                            errors.push(SemanticError {
                                message: format!("Domain '{}': Unknown type '{}' for field '{}'", domain.name, t, field.name),
                                span: to_span(field.span),
                            });
                        }

                        if state_fields.contains_key(&field.name) {
                            errors.push(SemanticError {
                                message: format!("Domain '{}': Duplicate state field '{}'", domain.name, field.name),
                                span: to_span(field.span),
                            });
                        } else {
                            state_fields.insert(field.name.clone(), v_type);        
                        }
                    }

                    if let Some(scope) = &domain.scope {
                        match scope {
                            crate::ast::AuthorityScope::Transition => {
                                if !domain.state.is_empty() || !domain.goals.is_empty() || !domain.invariants.is_empty() {
                                    errors.push(SemanticError {
                                        message: format!("Domain '{}' with scope 'transition' cannot declare state, goals, or invariants", domain.name),
                                        span: (0, 0).into(),
                                    });
                                }
                            }
                            crate::ast::AuthorityScope::Goal => {
                                if !domain.state.is_empty() || !domain.invariants.is_empty() {
                                    errors.push(SemanticError {
                                        message: format!("Domain '{}' with scope 'goal' cannot declare state or invariants", domain.name),
                                        span: (0, 0).into(),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }

                    let req_caps = domain.required_capabilities.clone();

                    self.domains.insert(domain.name.clone(), DomainInfo { 
                        state_fields,
                        required_capabilities: req_caps,
                        scope: domain.scope.clone(),
                    });
                }
                StatementKind::EnvironmentDecl(env) => {
                    declared_environments.push(env.name.clone());
                    
                    self.environment_capabilities.insert(env.name.clone(), env.available_capabilities.clone());
                    
                    let scope_str = match &env.scope_level {
                        Some(crate::ast::AuthorityScope::Root) => "root",
                        Some(crate::ast::AuthorityScope::Domain) => "domain",
                        Some(crate::ast::AuthorityScope::Goal) => "goal",
                        Some(crate::ast::AuthorityScope::Transition) => "transition",
                        None => "domain", // Default
                    };
                    self.environment_scopes.insert(env.name.clone(), scope_str.to_string());
                }
                _ => {}
            }
        }

        // Pass 2: Validate Deployments (Governance Rule E001)
        for stmt in &ast.statements {
            if let StatementKind::DeployStmt(deploy) = &stmt.kind {
                if !self.domains.contains_key(&deploy.service) { // Assuming service matches domain name for now
                    errors.push(SemanticError {
                        message: format!("E002: Unknown service '{}' in deployment statement.", deploy.service),
                        span: to_span(stmt.span),
                    });
                }
                if !declared_environments.contains(&deploy.target_environment) {
                    errors.push(SemanticError {
                        message: format!(
                            "E001: Execution authority violation\nManual context mutation detected.\n\nRequired: environment-bound capability\nFound: undeclared environment '{}'\n\nResolution:\nBind operation to a declared environment block.", deploy.target_environment
                        ),
                        span: to_span(stmt.span),
                    });
                }
            }
        }

        // Pass 3: Validate Goals and Transitions against State
        for stmt in &ast.statements {
            if let StatementKind::DomainDecl(domain) = &stmt.kind {
                let domain_info = self.domains.get(&domain.name).unwrap();      

                // Validate Goals (Strictly Pure)
                for goal in &domain.goals {
                    self.validate_expr(&domain.name, &goal.target, domain_info, true, &mut errors);
                }

                // Validate Invariants (Strictly Pure)
                for invariant in &domain.invariants {
                    self.validate_expr(&domain.name, &invariant.predicate, domain_info, true, &mut errors);
                }

                // Validate Transitions (Allow Mutations/Effects)
                for transition in &domain.transitions {
                    for expr in &transition.slice_step {
                        self.validate_expr(&domain.name, &expr, domain_info, false, &mut errors);
                    }
                    
                    // A007: Check transition required capabilities against domain bounds
                    let domain_caps = domain.required_capabilities.clone();
                    for req in &transition.required_capabilities {
                        if !domain_caps.contains(req) {
                            errors.push(SemanticError {
                                message: format!(
                                    "A007: Capability Escalation Violation\n\
                                     Transition '{}' in domain '{}' requires capability '{}'\n\
                                     which exceeds domain authority.\n\n\
                                     Domain provides: {:?}\n\
                                     Transition requires: {:?}",
                                    transition.name, domain.name, req,
                                    domain_caps, transition.required_capabilities
                                ),
                                span: to_span(transition.span),
                            });
                        }
                    }

                    // A008: Check that emitted effects have corresponding capabilities
                    let mut effects_found = Vec::new();
                    for expr in &transition.slice_step {
                        self.scan_effects_in_expr(expr, &mut effects_found);
                    }
                    
                    if !effects_found.is_empty() {
                        let has_messaging = transition.required_capabilities.contains(&"messaging".to_string());
                        if !has_messaging {
                            for (effect_type, span) in effects_found {
                                errors.push(SemanticError {
                                    message: format!(
                                        "A008: Effect Emission Capability Missing\n\
                                         Transition '{}' in domain '{}' emits '{}' effect\n\
                                         but lacks 'messaging' capability.\n\n\
                                         Transition capabilities: {:?}\n\
                                         Required to emit effects: \"messaging\"\n\n\
                                         Resolution:\nAdd 'messaging' to transition capabilities",
                                        transition.name, domain.name, effect_type,
                                        transition.required_capabilities
                                    ),
                                    span: to_span(span),
                                });
                            }
                        }
                    }
                }

                // Check goal required capabilities against domain bounds
                for goal in &domain.goals {
                    for req in &goal.required_capabilities {
                        if !domain.required_capabilities.contains(req) {
                            errors.push(SemanticError {
                                message: format!(
                                    "A007: Capability Escalation Violation\n\
                                     Goal '{}' in domain '{}' requires capability '{}'\n\
                                     which exceeds domain authority.\n\n\
                                     Domain provides: {:?}\n\
                                     Goal requires: {:?}",
                                    goal.name, domain.name, req,
                                    domain.required_capabilities, goal.required_capabilities
                                ),
                                span: to_span(goal.span),
                            });
                        }
                    }

                    // A008: Goals cannot emit effects (pure constraint)
                    let mut effects_found = Vec::new();
                    self.scan_effects_in_expr(&goal.target, &mut effects_found);
                    
                    for (effect_type, span) in effects_found {
                        errors.push(SemanticError {
                            message: format!(
                                "A008: Effect Emission in Pure Context\n\
                                 Goal '{}' in domain '{}' illegally emits '{}' effect.\n\
                                 Goals must be strictly pure and side-effect free.\n\n\
                                 Resolution:\nMove side effects to a transition",
                                goal.name, domain.name, effect_type
                            ),
                            span: to_span(span),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_expr(&self, domain_name: &str, expr: &Expr, domain_info: &DomainInfo, is_pure_context: bool, errors: &mut Vec<SemanticError>) {
        match &expr.kind {
            ExprKind::Ident(name) => {
                if !domain_info.state_fields.contains_key(name) {
                    errors.push(SemanticError {
                        message: format!("Domain '{}': Reference to undefined state variable '{}'", domain_name, name),
                        span: to_span(expr.span),
                    });
                }
            }
            ExprKind::Assignment { target, value } => {
                if is_pure_context {
                    errors.push(SemanticError {
                        message: format!("Domain '{}': Illegal mutation of '{}'. Goals must be strictly read-only and side-effect free.", domain_name, target),
                        span: to_span(expr.span),
                    });
                }
                if !domain_info.state_fields.contains_key(target) {
                    errors.push(SemanticError {
                        message: format!("Domain '{}': Cannot assign to undefined state variable '{}'", domain_name, target),
                        span: to_span(expr.span),
                    });
                }
                self.validate_expr(domain_name, value, domain_info, is_pure_context, errors);    
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.validate_expr(domain_name, left, domain_info, is_pure_context, errors);     
                self.validate_expr(domain_name, right, domain_info, is_pure_context, errors);    
            }
            ExprKind::If { condition, consequence } => {
                self.validate_expr(domain_name, condition, domain_info, is_pure_context, errors);
                for step in consequence {
                    self.validate_expr(domain_name, step, domain_info, is_pure_context, errors);
                }
            }
            ExprKind::While { condition, body } => {
                if let ExprKind::BoolLiteral(true) = condition.kind {
                    errors.push(SemanticError {
                        message: format!("Domain '{}': Unbounded `while(true)` loops are forbidden to preserve slice computation bounds.", domain_name),
                        span: to_span(expr.span),
                    });
                }
                
                self.validate_expr(domain_name, condition, domain_info, is_pure_context, errors);
                for step in body {
                    self.validate_expr(domain_name, step, domain_info, is_pure_context, errors);
                }
            }
            ExprKind::Send { target: _, message: _ } => {
                if is_pure_context {
                    errors.push(SemanticError {
                        message: format!("Domain '{}': Illegal effect 'send'. Goals must be strictly side-effect free.", domain_name),
                        span: to_span(expr.span),
                    });
                }
            }
            ExprKind::SendHigh { target: _, message: _ } => {
                if is_pure_context {
                    errors.push(SemanticError {
                        message: format!("Domain '{}': Illegal effect 'send_high'. Goals must be strictly side-effect free.", domain_name),
                        span: to_span(expr.span),
                    });
                }
            }
            ExprKind::Call { function, arguments } => {
                if function == "shell" {
                    errors.push(SemanticError {
                        message: format!(
                            "E001: Execution authority violation\nManual context mutation detected: `shell(...)` is forbidden.\n\nRequired: environment-bound capability\nFound: local session context\n\nResolution:\nBind operation to declared environment block."
                        ),
                        span: to_span(expr.span),
                    });
                }
                for arg in arguments {
                    self.validate_expr(domain_name, arg, domain_info, is_pure_context, errors);
                }
            }
            ExprKind::IntLiteral(_) | ExprKind::StringLiteral(_) | ExprKind::BoolLiteral(_) => {
                // Literals are inherently valid.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    #[test]
    fn test_valid_semantic() {
        let input = r#"
        domain WebServer {
            state {
                status: string
                port: int
            }
            goal is_running {
                target status == "online"
            }
            transition start_server {
                slice step {
                    status = "online"
                }
            }
        }
        "#;
        let ast = parse(lex(input)).unwrap();
        let mut validator = SemanticValidator::new();
        let result = validator.validate(&ast);
        assert!(result.is_ok(), "Should pass semantic validation");
    }

    #[test]
    fn test_invalid_semantic_variable() {
        let input = r#"
        domain WebServer {
            state {
                port: int
            }
            transition start_server {
                slice step {
                    status = "online"
                }
            }
        }
        "#;
        let ast = parse(lex(input)).unwrap();
        let mut validator = SemanticValidator::new();
        let result = validator.validate(&ast);
        assert!(result.is_err());
        let errors = result.err().unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("undefined state variable 'status'"));
    }

    #[test]
    fn test_invalid_goal_mutation() {
        let input = r#"
        domain WebServer {
            state {
                status: string
            }
            goal ensure_run {
                target status = "online"
            }
        }
        "#;
        let ast = parse(lex(input)).unwrap();
        let mut validator = SemanticValidator::new();
        let result = validator.validate(&ast);
        assert!(result.is_err());
        let errors = result.err().unwrap();
        assert!(errors.iter().any(|e| e.message.contains("Illegal mutation of 'status'")));
    }
}
