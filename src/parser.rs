use crate::lexer::{Token, Span};
use crate::ast::*;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code(ved::syntax))]
pub struct ParseError {
    pub message: String,
    #[label("expected token here")]
    pub span: SourceSpan,
}

pub struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, Span)>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &(Token, Span) {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens.last().unwrap()
        }
    }

    fn advance(&mut self) -> &(Token, Span) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn check(&self, expected: &Token) -> bool {
        &self.peek().0 == expected
    }

    fn consume(&mut self, expected: Token) -> Result<&(Token, Span), ParseError> {
        if self.check(&expected) {
            Ok(self.advance())
        } else {
            self.err(format!("Syntax Error at line {} col {}: Expected {}, but found {}", self.peek().1.line, self.peek().1.column, expected, self.peek().0))
        }
    }

        fn err<T>(&self, msg: String) -> Result<T, ParseError> {
        let span = self.peek().1;
        Err(ParseError {
            message: msg,
            span: (span.offset, span.len).into(),
        })
    }

    pub fn parse(&mut self) -> Result<Ast, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&Token::EOF) {
            match self.peek().0 {
                Token::Domain => statements.push(self.parse_domain()?),
                Token::System => statements.push(self.parse_system()?),
                Token::Environment => statements.push(self.parse_environment()?),
                Token::Deploy => statements.push(self.parse_deploy()?),
                _ => return self.err(format!("Unexpected token at top level: {}", self.peek().0)),
            }
        }

        Ok(Ast { statements })
    }

    fn parse_domain(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.consume(Token::Domain)?.1.clone();
        
        let name = match self.advance().0.clone() {
            Token::Identifier(id) => id,
            other => return self.err(format!("Expected identifier after 'domain', found {}", other)),
        };

        self.consume(Token::LBrace)?;

        let mut state = Vec::new();
        let mut goals = Vec::new();
        let mut transitions = Vec::new();
        let mut scope = None;
        let mut required_capabilities = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            match self.peek().0 {
                Token::Scope => {
                    self.consume(Token::Scope)?;
                    self.consume(Token::LBrace)?;
                    if let Token::Identifier(s) = self.advance().0.clone() {
                        match AuthorityScope::from_str(&s) {
                            Ok(auth) => scope = Some(auth),
                            Err(e) => return self.err(e),
                        }
                    } else {
                        return self.err("Expected identifier in domain scope".to_string());
                    }
                    self.consume(Token::RBrace)?;
                }
                Token::Capability => {
                    self.consume(Token::Capability)?;
                    self.consume(Token::LBrace)?;
                    while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
                        let tok = self.advance().0.clone();
                        if let Token::Identifier(id) = tok {
                            required_capabilities.push(id);
                        } else if tok != Token::Comma {
                            return self.err("Expected identifier or comma in capability list".to_string());
                        }
                    }
                    self.consume(Token::RBrace)?;
                }
                Token::State => state = self.parse_state_block()?,
                Token::Goal => goals.push(self.parse_goal()?),
                Token::Transition => transitions.push(self.parse_transition()?),
                _ => return self.err(format!("Unexpected token in domain body: {}", self.peek().0)),
            }
        }

        let rb = self.consume(Token::RBrace)?;

        let span = Span {
            offset: start_span.offset,
            len: rb.1.offset + rb.1.len - start_span.offset,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(Statement {
            kind: StatementKind::DomainDecl(DomainDecl { name, scope, required_capabilities, state, goals, transitions }),
            span,
        })
    }

    fn parse_state_block(&mut self) -> Result<Vec<StateField>, ParseError> {
        self.consume(Token::State)?;
        self.consume(Token::LBrace)?;
        let mut fields = Vec::new();
        
        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            let (name_tok, name_span) = self.advance().clone();
            let name = match name_tok {
                Token::Identifier(id) => id,
                other => return self.err(format!("Expected state field name, found {}", other)),
            };
            self.consume(Token::Colon)?;
            let (typ_tok, typ_span) = self.advance().clone();
            let typ = match typ_tok {
                Token::Identifier(id) => id,
                other => return self.err(format!("Expected type for field {}, found {}", name, other)),
            };
            
            let field_span = Span {
                offset: name_span.offset,
                len: typ_span.offset + typ_span.len - name_span.offset,
                line: name_span.line,
                column: name_span.column,
            };
            fields.push(StateField { name, typ, span: field_span });
        }
        self.consume(Token::RBrace)?;
        
        Ok(fields)
    }

    fn parse_goal(&mut self) -> Result<GoalDecl, ParseError> {
        let start_span = self.consume(Token::Goal)?.1.clone();
        let name = match self.advance().0.clone() {
            Token::Identifier(id) => id,
            other => return self.err(format!("Expected goal name, found {}", other)),
        };

        self.consume(Token::LBrace)?;
        
        let mut priority = 1; // Default priority
        let mut scope = None;
        let mut required_capabilities = Vec::new();

        while self.check(&Token::Scope) || self.check(&Token::Capability) || self.check(&Token::Priority) {
            match self.peek().0 {
                Token::Scope => {
                    self.consume(Token::Scope)?;
                    self.consume(Token::LBrace)?;
                    if let Token::Identifier(s) = self.advance().0.clone() {
                        match AuthorityScope::from_str(&s) {
                            Ok(auth) => scope = Some(auth),
                            Err(e) => return self.err(e),
                        }
                    } else {
                        return self.err("Expected identifier in goal scope".to_string());
                    }
                    self.consume(Token::RBrace)?;
                }
                Token::Capability => {
                    self.consume(Token::Capability)?;
                    self.consume(Token::LBrace)?;
                    while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
                        let tok = self.advance().0.clone();
                        if let Token::Identifier(id) = tok {
                            required_capabilities.push(id);
                        } else if tok != Token::Comma {
                            return self.err("Expected identifier or comma in capability list".to_string());
                        }
                    }
                    self.consume(Token::RBrace)?;
                }
                Token::Priority => {
                    self.consume(Token::Priority)?;
                    let tok = self.advance().0.clone();
                    if let Token::IntLiteral(p) = tok {
                        if p < 0 || p > 255 {
                            return self.err(format!("Priority must be between 0 and 255, found {}", p));
                        }
                        priority = p as u8;
                    } else {
                        return self.err(format!("Expected integer after 'priority', found {}", tok));
                    }
                }
                _ => break,
            }
        }

        if self.check(&Token::Target) {
            self.consume(Token::Target)?;
        } else if let Token::Identifier(ref id) = self.peek().0 {
            if id == "predicate" {
                self.advance();
            } else {
                return self.err(format!("Expected 'target' or 'predicate' for goal, found {}", self.peek().0));
            }
        } else {
            return self.err(format!("Expected 'target' or 'predicate' for goal, found {}", self.peek().0));
        }

        let target = self.parse_statement_or_expr()?;

        let mut strategy = Vec::new();

        if self.check(&Token::Strategy) {
            self.consume(Token::Strategy)?;
            self.consume(Token::LBrace)?;
            while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
                let tok = self.advance().0.clone();
                if let Token::Identifier(id) = tok {
                    strategy.push(id);
                } else if tok != Token::Comma {
                    // Ignore commas or gracefully fail on unexpected tokens
                }
            }
            self.consume(Token::RBrace)?;
        }

        let rb = self.consume(Token::RBrace)?;
        let span = Span {
            offset: start_span.offset,
            len: rb.1.offset + rb.1.len - start_span.offset,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(GoalDecl { name, scope, required_capabilities, target, strategy, priority, span })
    }

    fn parse_transition(&mut self) -> Result<TransitionDecl, ParseError> {
        let start_span = self.consume(Token::Transition)?.1.clone();
        let name = match self.advance().0.clone() {
            Token::Identifier(id) => id,
            other => return self.err(format!("Expected transition name, found {}", other)),
        };

        self.consume(Token::LBrace)?;
        let mut scope = None;
        let mut required_capabilities = Vec::new();

        while self.check(&Token::Scope) || self.check(&Token::Capability) {
            match self.peek().0 {
                Token::Scope => {
                    self.consume(Token::Scope)?;
                    self.consume(Token::LBrace)?;
                    if let Token::Identifier(s) = self.advance().0.clone() {
                        match AuthorityScope::from_str(&s) {
                            Ok(auth) => scope = Some(auth),
                            Err(e) => return self.err(e),
                        }
                    } else {
                        return self.err("Expected identifier in transition scope".to_string());
                    }
                    self.consume(Token::RBrace)?;
                }
                Token::Capability => {
                    self.consume(Token::Capability)?;
                    self.consume(Token::LBrace)?;
                    while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
                        let tok = self.advance().0.clone();
                        if let Token::Identifier(id) = tok {
                            required_capabilities.push(id);
                        } else if tok != Token::Comma {
                            return self.err("Expected identifier or comma in capability list".to_string());
                        }
                    }
                    self.consume(Token::RBrace)?;
                }
                _ => break,
            }
        }

        if self.check(&Token::Slice) {
            self.consume(Token::Slice)?;
        }
        self.consume(Token::Step)?;
        self.consume(Token::LBrace)?;
        
        let mut slice_step = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            slice_step.push(self.parse_statement_or_expr()?);
        }
        self.consume(Token::RBrace)?;
        let rb = self.consume(Token::RBrace)?;
        
        let span = Span {
            offset: start_span.offset,
            len: rb.1.offset + rb.1.len - start_span.offset,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(TransitionDecl { name, scope, required_capabilities, slice_step, span })
    }

    fn parse_environment(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.consume(Token::Environment)?.1.clone();
        let name = match self.advance().0.clone() {
            Token::Identifier(id) => id,
            _ => return self.err("Expected environment name".to_string()),
        };
        self.consume(Token::LBrace)?;
        
        let mut configurations = Vec::new();
        while !self.check(&Token::RBrace) && self.pos < self.tokens.len() {
            let key = match self.advance().0.clone() {
                Token::Identifier(id) => id,
                _ => return self.err("Expected configuration key".to_string()),
            };
            self.consume(Token::Equal)?;
            let value = self.parse_statement_or_expr()?;
            configurations.push((key, value));
        }
        let rb = self.consume(Token::RBrace)?;

        let span = Span {
            offset: start_span.offset,
            len: rb.1.offset + rb.1.len - start_span.offset,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(Statement {
            kind: StatementKind::EnvironmentDecl(EnvironmentDecl { name, configurations }),
            span,
        })
    }

    fn parse_deploy(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.consume(Token::Deploy)?.1.clone();
        
        let service = match self.advance().0.clone() {
            Token::Identifier(id) if id == "service" => {
                match self.advance().0.clone() {
                    Token::Identifier(svc_name) => svc_name,
                    _ => return self.err("Expected service name after 'deploy service'".to_string()),
                }
            },
            Token::Identifier(id) => id,
            _ => return self.err("Expected 'service' or identifier after 'deploy'".to_string()),
        };

        match self.advance().0.clone() {
            Token::Identifier(id) if id == "to" => id,
            Token::To => "to".to_string(),
            _ => return self.err("Expected 'to' in deploy statement".to_string()),
        };

        let (target_env_tok, target_span) = self.advance().clone();
        let target_environment = match target_env_tok {
            Token::Identifier(id) => id,
            _ => return self.err("Expected target environment name".to_string()),
        };

        let span = Span {
            offset: start_span.offset,
            len: target_span.offset + target_span.len - start_span.offset,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(Statement {
            kind: StatementKind::DeployStmt(DeployStmt { service, target_environment }),
            span,
        })
    }

    fn parse_system(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.consume(Token::System)?.1.clone();
        let name = match self.advance().0.clone() {
            Token::Identifier(id) => id,
            _ => return self.err("Expected system name".to_string()),
        };
        self.consume(Token::LBrace)?;
        
        let mut start_domains = Vec::new();
        while self.check(&Token::Start) {
            self.consume(Token::Start)?;
            self.consume(Token::Domain)?;
            let d_name = match self.advance().0.clone() {
                Token::Identifier(id) => id,
                _ => return self.err("Expected domain name".to_string()),
            };
            self.consume(Token::LBrace)?;
            let mut init_state = Vec::new();
            while !self.check(&Token::RBrace) {
                init_state.push(self.parse_statement_or_expr()?);
            }
            self.consume(Token::RBrace)?;
            start_domains.push(StartDomain { name: d_name, init_state });
        }
        let rb = self.consume(Token::RBrace)?;

        let span = Span {
            offset: start_span.offset,
            len: rb.1.offset + rb.1.len - start_span.offset,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(Statement {
            kind: StatementKind::SystemDecl(SystemDecl { name, start_domains }),
            span,
        })
    }

    // A Pratt-style parser for expressions
    fn parse_statement_or_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expression(0)
    }

    fn get_precedence(&self, token: &Token) -> u8 {
        match token {
            Token::Equal => 1,
            Token::EqualEqual | Token::NotEqual | Token::LessThan | Token::GreaterThan | Token::LTEqual | Token::GTEqual => 2,
            Token::Plus | Token::Minus => 3,
            Token::Asterisk | Token::Slash | Token::Modulo => 4,
            Token::LParen => 6, // Call operator precedence
            _ => 0,
        }
    }

    fn parse_expression(&mut self, precedence: u8) -> Result<Expr, ParseError> {
        let (token, mut start_span) = self.advance().clone();
        
        let mut left = match token {
            Token::IntLiteral(v) => Expr { kind: ExprKind::IntLiteral(v), span: start_span },
            Token::StringLiteral(s) => Expr { kind: ExprKind::StringLiteral(s), span: start_span },
            Token::Identifier(id) => Expr { kind: ExprKind::Ident(id), span: start_span },
            Token::True => Expr { kind: ExprKind::BoolLiteral(true), span: start_span },
            Token::False => Expr { kind: ExprKind::BoolLiteral(false), span: start_span },
            Token::LParen => {
                let inner = self.parse_expression(0)?;
                let rp = self.consume(Token::RParen)?;
                start_span.len = rp.1.offset + rp.1.len - start_span.offset;
                Expr { kind: inner.kind, span: start_span }
            }
            Token::Send => {
                self.consume(Token::LParen)?;
                let target = match self.advance().0.clone() {
                    Token::Identifier(id) => id,
                    Token::StringLiteral(s) => s,
                    other => return self.err(format!("Expected string/ident target for send, got {}", other)),
                };
                self.consume(Token::Comma)?;
                let message = match self.advance().0.clone() {
                    Token::Identifier(id) => id,
                    Token::StringLiteral(s) => s,
                    other => return self.err(format!("Expected string/ident msg for send, got {}", other)),
                };
                let rp = self.consume(Token::RParen)?;
                start_span.len = rp.1.offset + rp.1.len - start_span.offset;
                Expr { kind: ExprKind::Send { target, message }, span: start_span }
            }
            Token::SendHigh => {
                self.consume(Token::LParen)?;
                let target = match self.advance().0.clone() {
                    Token::Identifier(id) => id,
                    Token::StringLiteral(s) => s,
                    other => return self.err(format!("Expected string/ident target for send_high, got {}", other)),
                };
                self.consume(Token::Comma)?;
                let message = match self.advance().0.clone() {
                    Token::Identifier(id) => id,
                    Token::StringLiteral(s) => s,
                    other => return self.err(format!("Expected string/ident msg for send_high, got {}", other)),
                };
                let rp = self.consume(Token::RParen)?;
                start_span.len = rp.1.offset + rp.1.len - start_span.offset;
                Expr { kind: ExprKind::SendHigh { target, message }, span: start_span }
            }
            Token::If => {
                let condition = Box::new(self.parse_expression(0)?);
                self.consume(Token::LBrace)?;
                let mut consequence = Vec::new();
                while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
                    consequence.push(self.parse_statement_or_expr()?);
                }
                let rb = self.consume(Token::RBrace)?;
                start_span.len = rb.1.offset + rb.1.len - start_span.offset;
                Expr { kind: ExprKind::If { condition, consequence }, span: start_span }
            }
            Token::While => {
                let condition = Box::new(self.parse_expression(0)?);
                self.consume(Token::LBrace)?;
                let mut body = Vec::new();
                while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
                    body.push(self.parse_statement_or_expr()?);
                }
                let rb = self.consume(Token::RBrace)?;
                start_span.len = rb.1.offset + rb.1.len - start_span.offset;
                Expr { kind: ExprKind::While { condition, body }, span: start_span }
            }
            _ => return self.err(format!("Unexpected token in expression: {}", token)),
        };

        while precedence < self.get_precedence(&self.peek().0) {
            let (op_tok, _) = self.advance().clone();
            
            if op_tok == Token::LParen {
                let mut arguments = Vec::new();
                if !self.check(&Token::RParen) {
                    arguments.push(self.parse_expression(0)?);
                    while self.check(&Token::Comma) {
                        self.consume(Token::Comma)?;
                        arguments.push(self.parse_expression(0)?);
                    }
                }
                let rp = self.consume(Token::RParen)?;
                let span = Span {
                    offset: left.span.offset,
                    len: rp.1.offset + rp.1.len - left.span.offset,
                    line: left.span.line,
                    column: left.span.column,
                };
                let func_name = match left.kind {
                    ExprKind::Ident(id) => id,
                    _ => return self.err("Invalid function call target".to_string()),
                };
                left = Expr {
                    kind: ExprKind::Call { function: func_name, arguments },
                    span,
                };
                continue;
            }

            let op_str = match op_tok {
                Token::Plus => "+",
                Token::Minus => "-",
                Token::Asterisk => "*",
                Token::Slash => "/",
                Token::Modulo => "%",
                Token::EqualEqual => "==",
                Token::LessThan => "<",
                Token::GreaterThan => ">",
                Token::GTEqual => ">=",
                Token::LTEqual => "<=",
                Token::Equal => "=",
                _ => return self.err(format!("Unsupported binary operator: {:?}", op_tok)),
            }.to_string();

            let next_prec = self.get_precedence(&op_tok);
            
            // For right associative operations like '=', we would not add 1:
            let right_prec = if op_tok == Token::Equal { next_prec - 1 } else { next_prec };
            
            let right = self.parse_expression(right_prec)?;
            
            let span = Span {
                offset: left.span.offset,
                len: right.span.offset + right.span.len - left.span.offset,
                line: left.span.line,
                column: left.span.column,
            };

            if op_tok == Token::Equal {
                if let ExprKind::Ident(id) = left.kind {
                    left = Expr { kind: ExprKind::Assignment { target: id, value: Box::new(right) }, span };
                } else {
                    return self.err("Invalid assignment target".to_string());
                }
            } else {
                left = Expr {
                    kind: ExprKind::BinaryOp { left: Box::new(left), op: op_str, right: Box::new(right) },
                    span,
                };
            }
        }
        
        Ok(left)
    }
}

pub fn parse(input: Vec<(Token, Span)>) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn test_parse_domain_pseudocode() {
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
        
        let tokens = lex(input);
        let result = parse(tokens);
        
        assert!(result.is_ok(), "Failed to parse AST: {}", result.err().map(|e| e.message).unwrap_or_default());
        
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
        
        if let StatementKind::DomainDecl(domain) = &ast.statements[0].kind {
            assert_eq!(domain.name, "WebServer");
            assert_eq!(domain.state.len(), 2);
            assert_eq!(domain.state[0].name, "status");
            assert_eq!(domain.state[1].typ, "int");
            
            assert_eq!(domain.goals.len(), 1);
            assert_eq!(domain.goals[0].name, "is_running");
            
            assert_eq!(domain.transitions.len(), 1);
            assert_eq!(domain.transitions[0].name, "start_server");
        } else {
            panic!("Expected DomainDecl statement");
        }
    }
    
    #[test]
    fn test_precedence() {
        let input = "a + b * c == d";
        let mut tokens = lex(input);
        tokens.pop(); // remove EOF
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression(0).unwrap();
        // == is root
        if let ExprKind::BinaryOp { left, op, right } = expr.kind {
            assert_eq!(op, "==");
            if let ExprKind::Ident(r_name) = right.kind {
                assert_eq!(r_name, "d");
            } else { panic!("Right is not 'd'"); }
            
            if let ExprKind::BinaryOp { left: ll, op: mop, right: rr } = left.kind {
                assert_eq!(mop, "+");
                if let ExprKind::Ident(lname) = ll.kind { assert_eq!(lname, "a"); }
                
                if let ExprKind::BinaryOp { op: mmop, .. } = rr.kind {
                    assert_eq!(mmop, "*"); // b * c tight binding
                } else { panic!("Expected * operation"); }
            } else { panic!("Expected + operation"); }
        } else { panic!("Expected == operation"); }
    }
}
