use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenKind;

pub struct Parser {
    lexer: Lexer,
    curr: Option<Box<Token>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeKind {
    NUM(i64),
    ADD,
    SUB,
    MUL,
    DIV,
    Eq,
    NotEq,
    Leq,
    Lt,
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
        }
    }

    fn consume(&mut self) {
        self.curr = self.next_token();
    }

    pub fn run(&mut self) -> Result<Box<Node>, &'static str> {
        self.consume();
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> Result<Box<Node>, &'static str> {
        let node = self.parse_equality()?;
        Ok(node)
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
                TokenKind::Lit(s) => {
                    self.consume();
                    Ok(Box::new(Node::new_leaf(NodeKind::NUM(s.parse().unwrap()))))
                }
                _ => Err("unexpected token"),
            },
            None => Err("no new token"),
        }
    }

    fn next_token(&mut self) -> Option<Box<Token>> {
        self.lexer.next()
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
