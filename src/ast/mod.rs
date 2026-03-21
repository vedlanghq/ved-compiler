#[derive(Debug, Clone)]
pub struct SystemNode {
    pub name: String,
    pub domains: Vec<DomainNode>,
}

#[derive(Debug, Clone)]
pub struct DomainNode {
    pub name: String,
    pub state: StateNode,
    pub transitions: Vec<TransitionNode>,
    pub goals: Vec<GoalNode>,
}

#[derive(Debug, Clone)]
pub struct StateNode {
    pub fields: Vec<StateField>,
}

#[derive(Debug, Clone)]
pub struct StateField {
    pub name: String,
    pub field_type: String, // String representation of ty for now
    pub initial_value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct TransitionNode {
    pub name: String,
    pub body: BlockNode,
}

#[derive(Debug, Clone)]
pub struct GoalNode {
    pub condition: Expr,
    pub fallback_transition: String,
}

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment(String, Expr),
    EmitLog(String),
    SendMsg(String, String), // target_domain, transition_name
    TriggerMsg(String),      // target internal transition
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    String(String),
    Bool(bool),
    Identifier(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, Sub, Eq
}
