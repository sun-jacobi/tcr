use crate::lexer::Lexer;
use crate::lexer::Token;

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
            lhs: None, 
            rhs: Some(rhs),
        }
    }

    pub fn new_primary(kind: NodeKind) -> Self {
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
        let mut node = self.parse_mul()?;
        loop {
            match &self.curr {
                Some(token) => match **token {
                    Token::RESERVED('+') => {
                        self.consume();
                        let rhs = self.parse_mul()?;
                        node = Box::new(Node::new(NodeKind::ADD, node, rhs));
                    }
                    Token::RESERVED('-') => {
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
                Some(token) => match **token {
                    Token::RESERVED('*') => {
                        self.consume();
                        let rhs = self.parse_unary()?;
                        node = Box::new(Node::new(NodeKind::MUL, node, rhs));
                    }
                    Token::RESERVED('/') => {
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
            None => Err("No new token"),
            Some(token) => match **token {
                Token::RESERVED('+') => {
                    self.consume();
                    let rhs = self.parse_primary()?;
                    return Ok(Box::new(Node::new_unary(NodeKind::ADD, rhs)));
                }
                Token::RESERVED('-') => {
                    self.consume();
                    let rhs = self.parse_primary()?;
                    return Ok(Box::new(Node::new_unary(NodeKind::SUB, rhs)));
                } 
                _ => self.parse_primary(),
            }
           
        }
    }

    fn parse_primary(&mut self) -> Result<Box<Node>, &'static str> {
        match &self.curr {
            Some(token) => match **token {
                Token::RESERVED('(') => {
                    self.consume();
                    let node = self.parse_expr()?;
                    if !self.consume_char(')') {
                        return Err("invalid parentheses");
                    } else {
                        Ok(node)
                    }
                }
                Token::NUM(n) => {
                    self.consume();
                    Ok(Box::new(Node::new_primary(NodeKind::NUM(n))))
                }
                _ => Err("unexpected token"),
            },
            None => Err("no new token"),
        }
    }

    fn next_token(&mut self) -> Option<Box<Token>> {
        self.lexer.next()
    }

    fn consume_char(&mut self, expected: char) -> bool {
        match &self.curr {
            None => false,
            Some(token) => match **token {
                Token::RESERVED(actual) => {
                    if actual == expected {
                        self.consume();
                        return true;
                    } else {
                        false
                    }
                }
                _ => false,
            },
        }
    }
}
