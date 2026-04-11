use crate::lexer::Span;

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    DomainDecl(DomainDecl),
    SystemDecl(SystemDecl),
    EnvironmentDecl(EnvironmentDecl),
    DeployStmt(DeployStmt),
}

#[derive(Debug, Clone)]
pub struct DeployStmt {
    pub service: String,
    pub target_environment: String,
}

#[derive(Debug, Clone)]
pub struct EnvironmentDecl {
    pub name: String,
    pub configurations: Vec<(String, Expr)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthorityScope {
    Root,
    Domain,
    Goal,
    Transition,
}

impl AuthorityScope {
    pub fn from_str(id: &str) -> std::result::Result<Self, String> {
        match id.to_lowercase().as_str() {
            "root" => Ok(AuthorityScope::Root),
            "domain" => Ok(AuthorityScope::Domain),
            "goal" => Ok(AuthorityScope::Goal),
            "transition" => Ok(AuthorityScope::Transition),
            other => Err(format!("Unknown authority scope: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomainDecl {
    pub name: String,
    pub scope: Option<AuthorityScope>,
    pub required_capabilities: Vec<String>,
    pub state: Vec<StateField>,
    pub goals: Vec<GoalDecl>,
    pub transitions: Vec<TransitionDecl>,
}

#[derive(Debug, Clone)]
pub struct StateField {
    pub name: String,
    pub typ: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GoalDecl {
    pub name: String,
    pub scope: Option<AuthorityScope>,
    pub required_capabilities: Vec<String>,
    pub target: Expr,
    pub strategy: Vec<String>,
    pub priority: u8,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TransitionDecl {
    pub name: String,
    pub scope: Option<AuthorityScope>,
    pub required_capabilities: Vec<String>,
    pub slice_step: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SystemDecl {
    pub name: String,
    pub start_domains: Vec<StartDomain>,
}

#[derive(Debug, Clone)]
pub struct StartDomain {
    pub name: String,
    pub init_state: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Assignment {
        target: String,
        value: Box<Expr>,
    },
    Ident(String),
    IntLiteral(i64),
    StringLiteral(String),
    BoolLiteral(bool),
    Call {
        function: String,
        arguments: Vec<Expr>,
    },
    Send {
        target: String,
        message: String,
    },
    SendHigh {
        target: String,
        message: String,
    },
    If {
        condition: Box<Expr>,
        consequence: Vec<Expr>,
    },
    While {
        condition: Box<Expr>,
        body: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub statements: Vec<Statement>,
}
