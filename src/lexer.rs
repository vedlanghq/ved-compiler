#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub struct Span {
    pub offset: usize,
    pub len: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // System Structure Keywords
    Environment,
    Deploy,
    To,
    System,
    Module,
    Domain,
    State,
    Transition,
    Goal,
    Reconcile,
    Migration,

    // Execution & Scheduling
    Start,
    Step,
    Slice,
    Emit,
    Send,
    SendHigh,
    On,
    When,
    Target,
    Strategy,
    Priority,

    // Scope & Authority
    Scope,
    Elevate,
    Capability,

    // Visibility & Composition
    Public,
    Private,
    Import,

    // Control Flow
    If,
    Else,
    Let,
    While,

    // Literals
    True,
    False,

    // Symbols & Operators
    LBrace,      // {
    RBrace,      // }
    LParen,      // (
    RParen,      // )
    Colon,       // :
    Equal,       // =
    EqualEqual,  // ==
    NotEqual,    // !=
    LessThan,    // <
    GreaterThan, // >
    LTEqual,     // <=
    GTEqual,     // >=
    Arrow,       // ->
    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    Modulo,      // %
    Dot,         // .
    Comma,       // ,

    // Literals and Identifiers
    Identifier(String),
    IntLiteral(i64),
    StringLiteral(String),

    EOF,
    Unknown(char),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::System => write!(f, "'system'"),
            Token::Module => write!(f, "'module'"),
            Token::Domain => write!(f, "'domain'"),
            Token::State => write!(f, "'state'"),
            Token::Transition => write!(f, "'transition'"),
            Token::Goal => write!(f, "'goal'"),
            Token::Reconcile => write!(f, "'reconcile'"),
            Token::Migration => write!(f, "'migration'"),
            Token::Environment => write!(f, "'environment'"),
            Token::Deploy => write!(f, "'deploy'"),
            Token::Start => write!(f, "'start'"),
            Token::Step => write!(f, "'step'"),
            Token::Slice => write!(f, "'slice'"),
            Token::Emit => write!(f, "'emit'"),
            Token::Send => write!(f, "'send'"),
            Token::SendHigh => write!(f, "'send_high'"),
            Token::On => write!(f, "'on'"),
            Token::When => write!(f, "'when'"),
            Token::Target => write!(f, "'target'"),
            Token::Strategy => write!(f, "'strategy'"),
            Token::Priority => write!(f, "'priority'"),
            Token::Scope => write!(f, "'scope'"),
            Token::Elevate => write!(f, "'elevate'"),
            Token::Capability => write!(f, "'capability'"),
            Token::Public => write!(f, "'public'"),
            Token::Private => write!(f, "'private'"),
            Token::Import => write!(f, "'import'"),
            Token::If => write!(f, "'if'"),
            Token::Else => write!(f, "'else'"),
            Token::Let => write!(f, "'let'"),
            Token::While => write!(f, "'while'"),
            Token::True => write!(f, "'true'"),
            Token::False => write!(f, "'false'"),
            Token::LBrace => write!(f, "'{{'"),
            Token::RBrace => write!(f, "'}}'"),
            Token::LParen => write!(f, "'('"),
            Token::RParen => write!(f, "')'"),
            Token::Colon => write!(f, "':'"),
            Token::Equal => write!(f, "'='"),
            Token::EqualEqual => write!(f, "'=='"),
            Token::NotEqual => write!(f, "'!='"),
            Token::LessThan => write!(f, "'<'"),
            Token::GreaterThan => write!(f, "'>'"),
            Token::LTEqual => write!(f, "'<='"),
            Token::GTEqual => write!(f, "'>='"),
            Token::Arrow => write!(f, "'->'"),
            Token::Plus => write!(f, "'+'"),
            Token::Minus => write!(f, "'-'"),
            Token::Asterisk => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Modulo => write!(f, "'%'"),
            Token::Dot => write!(f, "'.'"),
            Token::Comma => write!(f, "','"),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::IntLiteral(n) => write!(f, "integer {}", n),
            Token::StringLiteral(s) => write!(f, "string \"{}\"", s),
            Token::To => write!(f, "'to'"),
            Token::EOF => write!(f, "end of file"),
            Token::Unknown(c) => write!(f, "unknown character '{}'", c),

        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { 
            input, 
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.position += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance_char();
            } else {
                break;
            }
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                self.advance_char();
            } else {
                break;
            }
        }
        let text = &self.input[start..self.position];
        match text {
            "environment" => Token::Environment,
            "deploy" => Token::Deploy,
            "to" => Token::To,
            "shell" => Token::Identifier("shell".to_string()),
            "system" => Token::System,
            "module" => Token::Module,
            "domain" => Token::Domain,
            "state" => Token::State,
            "transition" => Token::Transition,
            "goal" => Token::Goal,
            "reconcile" => Token::Reconcile,
            "migration" => Token::Migration,
            "start" => Token::Start,
            "step" => Token::Step,
            "slice" => Token::Slice,
            "emit" => Token::Emit,
            "send" => Token::Send,
            "send_high" => Token::SendHigh,
            "on" => Token::On,
            "when" => Token::When,
            "target" => Token::Target,
            "strategy" => Token::Strategy,
            "priority" => Token::Priority,
            "scope" => Token::Scope,
            "elevate" => Token::Elevate,
            "capability" => Token::Capability,
            "public" => Token::Public,
            "private" => Token::Private,
            "import" => Token::Import,
            "if" => Token::If,
            "else" => Token::Else,
            "let" => Token::Let,
            "while" => Token::While,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(text.to_string()),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        let text = &self.input[start..self.position];
        Token::IntLiteral(text.parse().unwrap_or(0))
    }

    fn read_string(&mut self) -> Token {
        self.advance_char(); // skip opening quote
        let start = self.position;
        while let Some(c) = self.peek_char() {
            if c == '"' {
                break;
            }
            self.advance_char();
        }
        let text = &self.input[start..self.position];
        self.advance_char(); // skip closing quote
        Token::StringLiteral(text.to_string())
    }

    pub fn next_token(&mut self) -> (Token, Span) {
        self.skip_whitespace();
        
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        let tok = match self.peek_char() {
            Some(c) => {
                if c.is_alphabetic() || c == '_' {
                    self.read_identifier_or_keyword()
                } else if c.is_ascii_digit() {
                    self.read_number()
                } else {
                    self.read_symbol(c)
                }
            }
            None => Token::EOF,
        };

        let span = Span {
            offset: start_pos,
            len: self.position - start_pos,
            line: start_line,
            column: start_col,
        };

        (tok, span)
    }

    fn read_symbol(&mut self, ch: char) -> Token {
        match ch {
            '{' => {
                self.advance_char();
                Token::LBrace
            }
            '}' => {
                self.advance_char();
                Token::RBrace
            }
            '(' => {
                self.advance_char();
                Token::LParen
            }
            ')' => {
                self.advance_char();
                Token::RParen
            }
            ':' => {
                self.advance_char();
                Token::Colon
            }
            '.' => {
                self.advance_char();
                Token::Dot
            }
            ',' => {
                self.advance_char();
                Token::Comma
            }
            '+' => {
                self.advance_char();
                Token::Plus
            }
            '-' => {
                self.advance_char();
                if self.peek_char() == Some('>') {
                    self.advance_char();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => {
                self.advance_char();
                Token::Asterisk
            }
            '/' => {
                self.advance_char();
                Token::Slash
            }
            '%' => {
                self.advance_char();
                Token::Modulo
            }
            '=' => {
                self.advance_char();
                if self.peek_char() == Some('=') {
                    self.advance_char();
                    Token::EqualEqual
                } else {
                    Token::Equal
                }
            }
            '!' => {
                self.advance_char();
                if self.peek_char() == Some('=') {
                    self.advance_char();
                    Token::NotEqual
                } else {
                    Token::Unknown('!')
                }
            }
            '<' => {
                self.advance_char();
                if self.peek_char() == Some('=') {
                    self.advance_char();
                    Token::LTEqual
                } else {
                    Token::LessThan
                }
            }
            '>' => {
                self.advance_char();
                if self.peek_char() == Some('=') {
                    self.advance_char();
                    Token::GTEqual
                } else {
                    Token::GreaterThan
                }
            }
            '"' => self.read_string(),
            _ => {
                self.advance_char();
                Token::Unknown(ch)
            }
        }
    }
}

pub fn lex(input: &str) -> Vec<(Token, Span)> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let (tok, span) = lexer.next_token();
        let is_eof = tok == Token::EOF;
        tokens.push((tok, span));
        if is_eof {
            break;
        }
    }
    tokens
}
