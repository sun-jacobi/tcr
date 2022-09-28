use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenKind;

pub struct Parser {
    lexer: Lexer,
    curr: Option<Box<Token>>,
    local: Vec<LVal>,
}

#[derive(Debug, PartialEq, Clone)]
struct LVal {
    name: String,
    offset: u8,
}

impl LVal {
    fn new(name: String, offset: u8) -> Self {
        Self { name, offset }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeKind {
    NUM(i64),
    LVAL(u8),
    ADD,
    SUB,
    MUL,
    DIV,
    If(Box<Node>),
    While,
    For {
        init: Box<Node>,
        end: Box<Node>,
        inc: Box<Node>,
    },
    Eq,
    NotEq,
    Leq,
    Lt,
    Nop,
    Assign,
    Return,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
        }
    }

    pub fn new_unary(kind: NodeKind, rhs: Box<Node>) -> Self {
        Self {
            kind,
            lhs: Some(Box::new(Node::new_leaf(NodeKind::NUM(0)))),
            rhs: Some(rhs),
        }
    }

    pub fn new_leaf(kind: NodeKind) -> Self {
        Self {
            kind,
            lhs: None,
            rhs: None,
        }
    }
}

impl Parser {
    pub fn load(src: String) -> Self {
        Self {
            lexer: Lexer::new(src),
            curr: None,
            local: Vec::new(),
        }
    }

    fn consume(&mut self) {
        self.curr = self.next_token();
    }
    #[allow(dead_code)]
    pub(crate) fn init(&mut self) {
        self.consume()
    }

    pub fn run(&mut self) -> Result<Vec<Box<Node>>, &'static str> {
        self.consume();
        self.parse_program()
    }

    fn parse_program(&mut self) -> Result<Vec<Box<Node>>, &'static str> {
        let mut code = Vec::new();
        loop {
            if let None = self.curr {
                return Ok(code);
            }
            let stmt = self.parse_stmt()?;
            code.push(stmt);
        }
    }

    pub fn parse_stmt(&mut self) -> Result<Box<Node>, &'static str> {
        // nop
        if self.consume_token(TokenKind::SemiCol) {
            return Ok(Box::new(Node::new_leaf(NodeKind::Nop)));
        }
        // if else
        if self.consume_token(TokenKind::If) {
            if !self.consume_token(TokenKind::OpenParen) {
                return Err("expected open parenthesis");
            }
            let expr = self.parse_expr()?;
            if !self.consume_token(TokenKind::CloseParen) {
                return Err("expected close parenthesis");
            }
            let lhs = self.parse_stmt()?;
            if self.consume_token(TokenKind::Else) {
                let rhs = self.parse_stmt()?;
                return Ok(Box::new(Node::new(NodeKind::If(expr), lhs, rhs)));
            }
            return Ok(Box::new(Node {
                kind: NodeKind::If(expr),
                lhs: Some(lhs),
                rhs: None,
            }));
        }
        // while statement
        if self.consume_token(TokenKind::While) {
            if !self.consume_token(TokenKind::OpenParen) {
                return Err("expected open parenthesis");
            }
            let expr = self.parse_expr()?;
            if !self.consume_token(TokenKind::CloseParen) {
                return Err("expected close parenthesis");
            }
            let stmt = self.parse_stmt()?;
            return Ok(Box::new(Node {
                kind: NodeKind::While,
                lhs: Some(expr),
                rhs: Some(stmt),
            }));
        }
        // for statement
        if self.consume_token(TokenKind::For) {
            if !self.consume_token(TokenKind::OpenParen) {
                return Err("expected open parenthesis");
            }
            let init = self.parse_stmt()?;
            let end = self.parse_stmt()?;
            let inc = self.parse_expr()?;
            if !self.consume_token(TokenKind::CloseParen) {
                return Err("expected close parenthesis");
            }
            let stmt = self.parse_stmt()?;
            return Ok(Box::new(Node {
                kind: NodeKind::For { init, end, inc },
                lhs: Some(stmt),
                rhs: None,
            }));
        }

        // return
        if self.consume_token(TokenKind::Return) {
            let expr = self.parse_expr()?;
            if !self.consume_token(TokenKind::SemiCol) {
                return Err("expected semicolon");
            }
            return Ok(Box::new(Node {
                kind: NodeKind::Return,
                lhs: None,
                rhs: Some(expr),
            }));
        }
        let expr = self.parse_expr()?;
        if !self.consume_token(TokenKind::SemiCol) {
            return Err("expected semicolon");
        }
        Ok(expr)
    }

    pub fn parse_expr(&mut self) -> Result<Box<Node>, &'static str> {
        let node = self.parse_assign()?;
        Ok(node)
    }

    fn parse_assign(&mut self) -> Result<Box<Node>, &'static str> {
        let mut node = self.parse_equality()?;
        loop {
            match &self.curr {
                None => return Ok(node),
                Some(token) => match token.kind {
                    TokenKind::Eq => {
                        self.consume();
                        let rhs = self.parse_assign()?;
                        node = Box::new(Node::new(NodeKind::Assign, node, rhs));
                    }
                    _ => return Ok(node),
                },
            }
        }
    }

    fn parse_equality(&mut self) -> Result<Box<Node>, &'static str> {
        let mut node = self.parse_relation()?;
        loop {
            match &self.curr {
                None => return Ok(node),
                Some(token) => match token.kind {
                    TokenKind::DoubleEq => {
                        self.consume();
                        let rhs = self.parse_relation()?;
                        node = Box::new(Node::new(NodeKind::Eq, node, rhs));
                    }
                    TokenKind::NotEq => {
                        self.consume();
                        let rhs = self.parse_relation()?;
                        node = Box::new(Node::new(NodeKind::NotEq, node, rhs));
                    }
                    _ => return Ok(node),
                },
            }
        }
    }

    fn parse_relation(&mut self) -> Result<Box<Node>, &'static str> {
        let mut node = self.parse_add()?;
        loop {
            match &self.curr {
                None => return Ok(node),
                Some(token) => match token.kind {
                    TokenKind::Geq => {
                        self.consume();
                        let rhs = self.parse_add()?;
                        node = Box::new(Node::new(NodeKind::Leq, rhs, node));
                    }
                    TokenKind::Gt => {
                        self.consume();
                        let rhs = self.parse_add()?;
                        node = Box::new(Node::new(NodeKind::Lt, rhs, node));
                    }
                    TokenKind::Leq => {
                        self.consume();
                        let rhs = self.parse_add()?;
                        node = Box::new(Node::new(NodeKind::Leq, node, rhs));
                    }
                    TokenKind::Lt => {
                        self.consume();
                        let rhs = self.parse_add()?;
                        node = Box::new(Node::new(NodeKind::Lt, node, rhs));
                    }
                    _ => return Ok(node),
                },
            }
        }
    }

    fn parse_add(&mut self) -> Result<Box<Node>, &'static str> {
        let mut node = self.parse_mul()?;
        loop {
            match &self.curr {
                Some(token) => match token.kind {
                    TokenKind::Add => {
                        self.consume();
                        let rhs = self.parse_mul()?;
                        node = Box::new(Node::new(NodeKind::ADD, node, rhs));
                    }
                    TokenKind::Minus => {
                        self.consume();
                        let rhs = self.parse_mul()?;
                        node = Box::new(Node::new(NodeKind::SUB, node, rhs));
                    }
                    _ => return Ok(node),
                },
                None => return Ok(node),
            }
        }
    }

    fn parse_mul(&mut self) -> Result<Box<Node>, &'static str> {
        let mut node = self.parse_unary()?;
        loop {
            match &self.curr {
                Some(token) => match token.kind {
                    TokenKind::Star => {
                        self.consume();
                        let rhs = self.parse_unary()?;
                        node = Box::new(Node::new(NodeKind::MUL, node, rhs));
                    }
                    TokenKind::Slash => {
                        self.consume();
                        let rhs = self.parse_unary()?;
                        node = Box::new(Node::new(NodeKind::DIV, node, rhs));
                    }
                    _ => return Ok(node),
                },
                None => return Ok(node),
            }
        }
    }

    fn parse_unary(&mut self) -> Result<Box<Node>, &'static str> {
        match &self.curr {
            None => return Err("No new token"),
            Some(token) => match token.kind {
                TokenKind::Add => {
                    self.consume();
                    let rhs = self.parse_unary()?;
                    return Ok(Box::new(Node::new_unary(NodeKind::ADD, rhs)));
                }
                TokenKind::Minus => {
                    self.consume();
                    let rhs = self.parse_unary()?;
                    return Ok(Box::new(Node::new_unary(NodeKind::SUB, rhs)));
                }
                _ => return self.parse_primary(),
            },
        }
    }

    fn parse_primary(&mut self) -> Result<Box<Node>, &'static str> {
        match &self.curr {
            Some(token) => match token.kind.to_owned() {
                TokenKind::OpenParen => {
                    self.consume();
                    let node = self.parse_expr()?;
                    if !self.consume_token(TokenKind::CloseParen) {
                        return Err("invalid parentheses");
                    } else {
                        Ok(node)
                    }
                }
                TokenKind::Num(s) => {
                    self.consume();
                    Ok(Box::new(Node::new_leaf(NodeKind::NUM(s.parse().unwrap()))))
                }
                TokenKind::Ident(name) => {
                    self.consume();
                    if let Some(offset) = self.find_lval(&name) {
                        return Ok(Box::new(Node::new_leaf(NodeKind::LVAL(offset))));
                    }
                    if let None = self.local.last() {
                        self.local.push(LVal::new(name, 8));
                        return Ok(Box::new(Node::new_leaf(NodeKind::LVAL(8))));
                    }
                    let offset = self.local.last().unwrap().offset + 8;
                    self.local.push(LVal::new(name, offset));
                    Ok(Box::new(Node::new_leaf(NodeKind::LVAL(offset))))
                }
                _ => Err("unexpected token"),
            },
            None => Err("no new token"),
        }
    }

    fn next_token(&mut self) -> Option<Box<Token>> {
        self.lexer.next()
    }
    fn find_lval(&self, ident: &str) -> Option<u8> {
        let lval = self.local.iter().find(|lval| lval.name == ident)?;
        Some(lval.offset)
    }

    fn consume_token(&mut self, expected: TokenKind) -> bool {
        match &self.curr {
            None => false,
            Some(actual) => {
                if actual.kind == expected {
                    self.consume();
                    return true;
                }
                false
            }
        }
    }
}
