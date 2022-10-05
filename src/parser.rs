use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::lexer::TokenKind;

pub(crate) struct Parser {
    lexer: Lexer,
    curr: Option<Box<Token>>,
    local: Vec<Vec<LVal>>, // local frames for functions
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Type {
    INT,
    PTR(Box<Type>),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct LVal {
    name: String,
    val_type: Type,
    offset: u8,
}

impl LVal {
    fn new(name: String, val_type: Type, offset: u8) -> Self {
        Self {
            name,
            val_type,
            offset,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum NodeKind {
    NUM(i64),
    LVAL(u8),
    ADD,
    SUB,
    MUL,
    DIV,
    Deref, // *
    Addr,  // &
    Block(Vec<Box<Node>>),
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
    Func {
        name: String,
        argv: Vec<Box<Node>>,
    }, // function call
    Def {
        name: String,
        args: usize,
        body: Box<Node>,
        local: usize, // index in the local frames
    }, // function definition
    Declar, // define new variable
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Node {
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

    pub fn run(&mut self) -> Result<Vec<Box<Node>>, String> {
        self.consume();
        self.parse_program()
    }

    fn parse_program(&mut self) -> Result<Vec<Box<Node>>, String> {
        let mut code = Vec::new();
        loop {
            if let None = self.curr {
                return Ok(code);
            }
            if self.consume_token(TokenKind::Int) {
                let function = self.parse_def()?;
                code.push(function);
            } else {
                return Err(String::from("expected function."));
            }
        }
    }

    fn parse_def(&mut self) -> Result<Box<Node>, String> {
        match &self.curr {
            None => return Err("no new token".to_string()),
            Some(token) => match token.kind.to_owned() {
                TokenKind::Ident(name) => {
                    self.consume();
                    self.parse_func(name)
                }
                _ => return Err("expected function name".to_string()),
            },
        }
    }

    fn parse_func(&mut self, name: String) -> Result<Box<Node>, String> {
        let local = self.local.len();
        self.local.push(Vec::new());
        let args = self.parse_args()?;
        if !self.peek_token(TokenKind::OpenCur) {
            return Err("expected function body.".to_string());
        }
        let body = self.parse_stmt()?;
        Ok(Box::new(Node::new_leaf(NodeKind::Def {
            name,
            args,
            body,
            local,
        })))
    }

    fn parse_arg(&mut self, val_type: Type) -> Result<(), String> {
        match &self.curr {
            None => return Err("expected argument".to_string()),
            Some(token) => match token.kind.to_owned() {
                TokenKind::Ident(arg) => {
                    self.consume();
                    self.push_local(val_type, arg.clone());
                    Ok(())
                }
                _ => return Err("expected argument".to_string()),
            },
        }
    }

    fn parse_ptr(&mut self, init_type: Type) -> Type {
        let mut val_type = init_type;
        loop {
            if self.consume_token(TokenKind::Star) {
                val_type = Type::PTR(Box::new(val_type));
            } else {
                return val_type;
            }
        }
    }

    fn parse_args(&mut self) -> Result<usize, String> {
        if !self.consume_token(TokenKind::OpenParen) {
            return Err("expected open parenthesis.".to_string());
        }
        let mut args = 0;
        if self.consume_token(TokenKind::CloseParen) {
            return Ok(args);
        }
        loop {
            match &self.curr {
                None => return Err("no new token".to_string()),
                Some(token) => match token.kind.to_owned() {
                    TokenKind::Int => {
                        self.consume();
                        let val_type = self.parse_ptr(Type::INT);
                        args += 1;
                        self.parse_arg(val_type)?;
                        if self.consume_token(TokenKind::CloseParen) {
                            return Ok(args);
                        }
                        if self.consume_token(TokenKind::Comma) {
                            continue;
                        }
                    }
                    _ => return Err("unexpected token".to_string()),
                },
            }
        }
    }

    fn parse_var(&mut self, val_type: Type) -> Result<Box<Node>, String> {
        match &self.curr {
            None => return Err("expected variable name".to_string()),
            Some(token) => match token.kind.to_owned() {
                TokenKind::Ident(name) => {
                    self.consume();
                    let offset = self.push_local(val_type, name.clone());
                    if self.consume_token(TokenKind::SemiCol) {
                        return Ok(Box::new(Node {
                            kind: NodeKind::Declar,
                            lhs: Some(Box::new(Node::new_leaf(NodeKind::LVAL(offset)))),
                            rhs: None,
                        }));
                    } else {
                        return Err("TODO".to_string());
                    }
                }
                _ => return Err("expected variable name".to_string()),
            },
        }
    }

    fn parse_stmt(&mut self) -> Result<Box<Node>, String> {
        // declare new lval
        if self.consume_token(TokenKind::Int) {
            let val_type = self.parse_ptr(Type::INT);
            return self.parse_var(val_type);
        }

        // nop
        if self.consume_token(TokenKind::SemiCol) {
            return Ok(Box::new(Node::new_leaf(NodeKind::Nop)));
        }

        // Block
        if self.consume_token(TokenKind::OpenCur) {
            let mut stmts = Vec::new();
            loop {
                match &self.curr {
                    None => return Err("no new token".to_string()),
                    Some(token) => match token.kind {
                        TokenKind::CloseCur => {
                            self.consume();
                            return Ok(Box::new(Node::new_leaf(NodeKind::Block(stmts))));
                        }
                        _ => stmts.push(self.parse_stmt()?),
                    },
                }
            }
        }
        // if else
        if self.consume_token(TokenKind::If) {
            if !self.consume_token(TokenKind::OpenParen) {
                return Err("expected open parenthesis".to_string());
            }
            let expr = self.parse_expr()?;
            if !self.consume_token(TokenKind::CloseParen) {
                return Err("expected close parenthesis".to_string());
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
                return Err("expected open parenthesis".to_string());
            }
            let expr = self.parse_expr()?;
            if !self.consume_token(TokenKind::CloseParen) {
                return Err("expected close parenthesis".to_string());
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
                return Err("expected open parenthesis".to_string());
            }
            let init = if self.peek_token(TokenKind::SemiCol) {
                Box::new(Node::new_leaf(NodeKind::Nop))
            } else {
                self.parse_expr()?
            };
            if !self.consume_token(TokenKind::SemiCol) {
                return Err("expected semicolon".to_string());
            }
            let end = if self.peek_token(TokenKind::SemiCol) {
                Box::new(Node::new_leaf(NodeKind::Nop))
            } else {
                self.parse_expr()?
            };
            if !self.consume_token(TokenKind::SemiCol) {
                return Err("expected semicolon".to_string());
            }
            let inc = if self.peek_token(TokenKind::SemiCol) {
                Box::new(Node::new_leaf(NodeKind::Nop))
            } else {
                self.parse_expr()?
            };
            if !self.consume_token(TokenKind::CloseParen) {
                return Err("expected close parenthesis".to_string());
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
                return Err("expected semicolon".to_string());
            }
            return Ok(Box::new(Node {
                kind: NodeKind::Return,
                lhs: None,
                rhs: Some(expr),
            }));
        }
        let expr = self.parse_expr()?;
        if !self.consume_token(TokenKind::SemiCol) {
            return Err("expected semicolon".to_string());
        }
        Ok(expr)
    }

    fn parse_expr(&mut self) -> Result<Box<Node>, String> {
        let node = self.parse_assign()?;
        Ok(node)
    }

    fn parse_assign(&mut self) -> Result<Box<Node>, String> {
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

    fn parse_equality(&mut self) -> Result<Box<Node>, String> {
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

    fn parse_relation(&mut self) -> Result<Box<Node>, String> {
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

    fn parse_add(&mut self) -> Result<Box<Node>, String> {
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

    fn parse_mul(&mut self) -> Result<Box<Node>, String> {
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

    fn parse_unary(&mut self) -> Result<Box<Node>, String> {
        match &self.curr {
            None => return Err("No new token".to_string()),
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
                TokenKind::Star => {
                    self.consume();
                    let rhs = self.parse_primary()?;
                    if let NodeKind::LVAL(_) = rhs.kind {
                        return Ok(Box::new(Node::new_unary(NodeKind::Deref, rhs)));
                    } else {
                        return Err("expected lval".to_string());
                    }
                }
                TokenKind::And => {
                    self.consume();
                    let rhs = self.parse_primary()?;
                    if let NodeKind::LVAL(_) = rhs.kind {
                        return Ok(Box::new(Node::new_unary(NodeKind::Addr, rhs)));
                    } else {
                        return Err("expected lval".to_string());
                    }
                }
                _ => return self.parse_primary(),
            },
        }
    }

    fn parse_primary(&mut self) -> Result<Box<Node>, String> {
        match &self.curr {
            Some(token) => match token.kind.to_owned() {
                TokenKind::OpenParen => {
                    self.consume();
                    let node = self.parse_expr()?;
                    if !self.consume_token(TokenKind::CloseParen) {
                        return Err("invalid parentheses".to_string());
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
                    // function call
                    if self.consume_token(TokenKind::OpenParen) {
                        let mut argv = Vec::new();
                        if self.consume_token(TokenKind::CloseParen) {
                            return Ok(Box::new(Node::new_leaf(NodeKind::Func { name, argv })));
                        }
                        loop {
                            let arg = self.parse_expr()?;
                            argv.push(arg);
                            if self.consume_token(TokenKind::CloseParen) {
                                return Ok(Box::new(Node::new_leaf(NodeKind::Func { name, argv })));
                            }
                            if self.consume_token(TokenKind::Comma) {
                                continue;
                            }
                        }
                    }

                    if let Some(offset) = self.find_lval(&name) {
                        return Ok(Box::new(Node::new_leaf(NodeKind::LVAL(offset))));
                    } else {
                        return Err("variable not defined".to_string());
                    }
                }
                _ => Err("unexpected token".to_string()),
            },
            None => Err("no new token".to_string()),
        }
    }

    fn next_token(&mut self) -> Option<Box<Token>> {
        self.lexer.next()
    }
    fn find_lval(&self, ident: &str) -> Option<u8> {
        let cur_local = self.local.last().unwrap();
        let lval = cur_local.iter().find(|lval| lval.name == ident)?;
        Some(lval.offset)
    }

    fn peek_token(&mut self, expected: TokenKind) -> bool {
        match &self.curr {
            None => false,
            Some(actual) => {
                if actual.kind == expected {
                    return true;
                }
                false
            }
        }
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

    pub fn get_local_size(&self, id: usize) -> usize {
        self.local[id].len()
    }

    fn push_local(&mut self, val_type: Type, name: String) -> u8 {
        let id = self.local.len() - 1;
        if self.local[id].len() == 0 {
            self.local[id].push(LVal::new(name, val_type, 8));
            return 8;
        } else {
            let offset = self.local[id].last().unwrap().offset + 8;
            self.local[id].push(LVal::new(name, val_type, offset));
            return offset;
        }
    }
}

//------------------------------------------------------------------------
//------------------------------------------------------------------------

#[cfg(test)]
impl Parser {
    fn new_stack(&mut self) {
        self.local.push(Vec::new());
    }
    fn init(&mut self) {
        self.consume();
        self.new_stack();
    }
}

#[test]
fn add_test() {
    let code = String::from("42 + 31");
    let mut parser = Parser::load(code);
    parser.consume();
    // root for an ast
    let root = parser.parse_expr().unwrap();
    assert_eq!(root.kind, NodeKind::ADD);
    assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(42));
    assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(31));
}

#[test]
fn single_test() {
    let code = String::from("42");
    let mut parser = Parser::load(code);
    parser.consume();
    // root for an ast
    let root = parser.parse_expr().unwrap();
    assert_eq!(root.kind, NodeKind::NUM(42));
}

#[test]
fn mul_test() {
    let code = String::from("42*31");
    let mut parser = Parser::load(code);
    parser.consume();
    // root for an ast
    let root = parser.parse_expr().unwrap();
    assert_eq!(root.kind, NodeKind::MUL);
    assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(42));
    assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(31));
}

#[test]
fn addmul_test() {
    let code = String::from("42 + 31 * 1");
    let mut parser = Parser::load(code);
    parser.consume();
    // root for an ast
    let root = parser.parse_expr().unwrap();
    let lhs = root.lhs.unwrap();
    let rhs = root.rhs.unwrap();
    assert_eq!(root.kind, NodeKind::ADD);
    assert_eq!(rhs.kind, NodeKind::MUL);
    assert_eq!(lhs.kind, NodeKind::NUM(42));
    assert_eq!(rhs.lhs.unwrap().kind, NodeKind::NUM(31));
    assert_eq!(rhs.rhs.unwrap().kind, NodeKind::NUM(1));
}

#[test]
fn bracket_test() {
    let code = String::from("42 * (31 + 1)");
    let mut parser = Parser::load(code);
    parser.consume();
    // root for an ast
    let root = parser.parse_expr().unwrap();
    let lhs = root.lhs.unwrap();
    let rhs = root.rhs.unwrap();
    assert_eq!(root.kind, NodeKind::MUL);
    assert_eq!(rhs.kind, NodeKind::ADD);
    assert_eq!(lhs.kind, NodeKind::NUM(42));
    assert_eq!(rhs.lhs.unwrap().kind, NodeKind::NUM(31));
    assert_eq!(rhs.rhs.unwrap().kind, NodeKind::NUM(1));
}

#[test]
fn unary_test() {
    let code = String::from("-42 * +31");
    let mut parser = Parser::load(code);
    parser.consume();
    // root for an ast
    let root = parser.parse_expr().unwrap();
    let lhs = root.lhs.unwrap();
    let rhs = root.rhs.unwrap();
    assert_eq!(root.kind, NodeKind::MUL);
    assert_eq!(lhs.kind, NodeKind::SUB);
    assert_eq!(rhs.kind, NodeKind::ADD);
    assert_eq!(lhs.lhs.unwrap().kind, NodeKind::NUM(0));
    assert_eq!(lhs.rhs.unwrap().kind, NodeKind::NUM(42));
    assert_eq!(rhs.lhs.unwrap().kind, NodeKind::NUM(0));
    assert_eq!(rhs.rhs.unwrap().kind, NodeKind::NUM(31));
}

#[test]
fn simple_relation_test() {
    let code = String::from("42  >=   31 ");
    let mut parser = Parser::load(code);
    parser.consume();
    let root = parser.parse_expr().unwrap();
    assert_eq!(root.kind, NodeKind::Leq);
    assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(42));
    assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(31));
}

#[test]
fn two_relation_test() {
    let code = String::from("42 * 31 >=   31 + 42");
    let mut parser = Parser::load(code);
    parser.consume();
    let root = parser.parse_expr().unwrap();
    let lhs = root.lhs.unwrap();
    let rhs = root.rhs.unwrap();
    assert_eq!(root.kind, NodeKind::Leq);
    assert_eq!(lhs.kind, NodeKind::ADD);
    assert_eq!(rhs.kind, NodeKind::MUL);
    assert_eq!(lhs.lhs.unwrap().kind, NodeKind::NUM(31));
    assert_eq!(lhs.rhs.unwrap().kind, NodeKind::NUM(42));
    assert_eq!(rhs.lhs.unwrap().kind, NodeKind::NUM(42));
    assert_eq!(rhs.rhs.unwrap().kind, NodeKind::NUM(31));
}

#[test]
fn stmt_test() {
    let code = "{int a; a = 42;}".to_string();
    let mut parser = Parser::load(code);
    parser.consume();
    parser.new_stack();
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Block(stmts) = node.kind {
        assert_eq!(stmts.len(), 2);
        let first = stmts[0].clone();
        let second = stmts[1].clone();
        assert_eq!(first.kind, NodeKind::Declar);
        assert_eq!(first.lhs.unwrap().kind, NodeKind::LVAL(8));
        assert_eq!(first.rhs, None);
        assert_eq!(second.kind, NodeKind::Assign);
        assert_eq!(second.lhs.unwrap().kind, NodeKind::LVAL(8));
        assert_eq!(second.rhs.unwrap().kind, NodeKind::NUM(42));
    } else {
        panic!("expect block statement");
    }
}

#[test]
fn assign_test() {
    let code = "{a = 42; b = 31; a = 31;}".to_string();
    let mut parser = Parser::load(code);
    parser.consume();
    parser.new_stack();
    parser.push_local(Type::INT, "a".to_string());
    parser.push_local(Type::INT, "b".to_string());
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Block(stmts) = node.kind {
        assert_eq!(stmts.len(), 3);
        let first = stmts[0].clone();
        let second = stmts[1].clone();
        let third = stmts[2].clone();
        assert_eq!(first.kind, NodeKind::Assign);
        assert_eq!(first.lhs.unwrap().kind, NodeKind::LVAL(8));
        assert_eq!(first.rhs.unwrap().kind, NodeKind::NUM(42));
        assert_eq!(second.kind, NodeKind::Assign);
        assert_eq!(second.lhs.unwrap().kind, NodeKind::LVAL(16));
        assert_eq!(second.rhs.unwrap().kind, NodeKind::NUM(31));
        assert_eq!(third.kind, NodeKind::Assign);
        assert_eq!(third.lhs.unwrap().kind, NodeKind::LVAL(8));
        assert_eq!(third.rhs.unwrap().kind, NodeKind::NUM(31));
    } else {
        panic!("expect block statement");
    }
}

#[test]
fn return_test() {
    let code = String::from("return 42;");
    let mut parser = Parser::load(code);
    parser.consume();
    let root = parser.parse_stmt().unwrap();
    assert_eq!(root.kind, NodeKind::Return);
    assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(42));
}

#[test]
fn if_simple_test() {
    let code = String::from("if (42) return 42;");
    let mut parser = Parser::load(code);
    parser.consume();
    let stmt = parser.parse_stmt().unwrap();
    if let NodeKind::If(expr) = stmt.kind {
        assert_eq!(expr.kind, NodeKind::NUM(42));
    } else {
        panic!("expected if statement");
    }
    let lhs = stmt.lhs.unwrap();
    let rhs = stmt.rhs;
    assert_eq!(lhs.kind, NodeKind::Return);
    assert_eq!(rhs, None);
}
#[test]
fn if_else_test() {
    let code = String::from("if (42) return 42; else return 31;");
    let mut parser = Parser::load(code);
    parser.consume();
    let stmt = parser.parse_stmt().unwrap();
    if let NodeKind::If(expr) = stmt.kind {
        assert_eq!(expr.kind, NodeKind::NUM(42));
    } else {
        panic!("expected if statement");
    }
    let lhs = stmt.lhs.unwrap();
    let rhs = stmt.rhs.unwrap();
    assert_eq!(lhs.kind, NodeKind::Return);
    assert_eq!(rhs.kind, NodeKind::Return);
}

#[test]
fn for_test() {
    let code = String::from("for(a=2; a <= 4; a = a + 1) ;");
    let mut parser = Parser::load(code);
    parser.consume();
    parser.new_stack();
    parser.push_local(Type::INT, "a".to_string());
    let stmt = parser.parse_stmt().unwrap();
    if let NodeKind::For { init, end, inc } = stmt.kind {
        assert_eq!(init.kind, NodeKind::Assign);
        assert_eq!(end.kind, NodeKind::Leq);
        assert_eq!(inc.kind, NodeKind::Assign);
    } else {
        panic!("expected statement");
    }
    let lhs = stmt.lhs.unwrap();
    assert_eq!(lhs.kind, NodeKind::Nop);
}
#[test]
fn while_test() {
    let code = String::from("while(42) ;");
    let mut parser = Parser::load(code);
    parser.consume();
    parser.new_stack();
    let node = parser.parse_stmt().unwrap();
    let lhs = node.lhs.unwrap();
    let rhs = node.rhs.unwrap();
    assert_eq!(node.kind, NodeKind::While);
    assert_eq!(lhs.kind, NodeKind::NUM(42));
    assert_eq!(rhs.kind, NodeKind::Nop);
}

#[test]
fn block_test() {
    let code = String::from("{42; 31;}");
    let mut parser = Parser::load(code);
    parser.consume();
    parser.new_stack();
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Block(block) = node.kind {
        assert_eq!(block.len(), 2);
        let first = block[0].clone();
        let second = block[1].clone();
        assert_eq!(first.kind, NodeKind::NUM(42));
        assert_eq!(second.kind, NodeKind::NUM(31));
    } else {
        panic!("expected block");
    }
}
#[test]
fn if_block_test() {
    let code = String::from("if (a > 1) {42;}");
    let mut parser = Parser::load(code);
    parser.consume();
    parser.new_stack();
    parser.push_local(Type::INT, "a".to_string());
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::If(_) = node.kind {
        let block = node.lhs.unwrap();
        if let NodeKind::Block(stmt) = block.kind {
            assert_eq!(stmt.len(), 1);
        } else {
            panic!("expected block");
        }
    } else {
        panic!("expected if statement");
    }
}

#[test]
fn func_test() {
    let code = String::from("foo();");
    let mut parser = Parser::load(code);
    parser.consume();
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Func { name, argv } = node.kind {
        assert_eq!(name, "foo");
        assert_eq!(argv.len(), 0);
    } else {
        panic!("expecten function call");
    }
}

#[test]
fn func_mul_test() {
    let code = String::from("foo(42, 31);");
    let mut parser = Parser::load(code);
    parser.consume();
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Func { name, argv } = node.kind {
        assert_eq!(name, "foo");
        assert_eq!(argv.len(), 2);
        assert_eq!(argv[0].kind, NodeKind::NUM(42));
        assert_eq!(argv[1].kind, NodeKind::NUM(31));
    } else {
        panic!("expecten function call");
    }
}

#[test]
fn def_test() {
    let code = String::from("int foo(int a, int b, int c){return a + b + c;}");
    let mut parser = Parser::load(code);
    let functions = parser.run().unwrap();
    assert_eq!(functions.len(), 1);
    let foo = functions[0].clone();
    if let NodeKind::Def {
        name,
        args,
        body,
        local,
    } = foo.kind
    {
        assert_eq!(name, "foo");
        assert_eq!(3, parser.local[0].len());
        assert_eq!(args, 3);
        assert_eq!(local, 0);
        //assert_eq!(argv[0].name, "a");
        //assert_eq!(argv[1].name, "b");
        if let NodeKind::Block(stmts) = body.kind {
            assert_eq!(stmts.len(), 1);
            let stmt = stmts[0].clone();
            assert_eq!(stmt.kind, NodeKind::Return);
        } else {
            panic!("expected function definition")
        }
    } else {
        panic!("expected function definition")
    }
}

#[test]
fn ptr_test() {
    let code = String::from("{int a; a = 2; int b; b = &a;}");
    let mut parser = Parser::load(code);
    parser.init();
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Block(stmts) = node.kind {
        let forth = stmts[3].clone();
        let rhs = forth.rhs.unwrap();
        assert_eq!(forth.kind, NodeKind::Assign);
        assert_eq!(rhs.kind, NodeKind::Addr);
    } else {
        panic!("expected block")
    }
}

#[test]
fn deref_test() {
    let code = String::from("{int a; a = 2; int b; b = *a;}");
    let mut parser = Parser::load(code);
    parser.init();
    let node = parser.parse_stmt().unwrap();
    if let NodeKind::Block(stmts) = node.kind {
        let forth = stmts[3].clone();
        let rhs = forth.rhs.unwrap();
        assert_eq!(forth.kind, NodeKind::Assign);
        assert_eq!(rhs.kind, NodeKind::Deref);
    } else {
        panic!("expected block")
    }
}

#[test]
fn dec_test() {
    let code = String::from("int main(){int a; a = 2;}");
    let mut parser = Parser::load(code);
    let functions = parser.run().unwrap();
    assert_eq!(functions.len(), 1);
    let func = functions[0].clone();
    if let NodeKind::Def {
        name: _,
        args: _,
        body: _d,
        local,
    } = func.kind
    {
        assert_eq!(parser.get_local_size(local), 1);
        assert_eq!(parser.local[0][0].name, "a");
    } else {
        panic!("expected function def");
    }
}

#[test]
#[should_panic]
fn not_find_var() {
    let code = String::from("int main(){int a; b = 2;}");
    let mut parser = Parser::load(code);
    let _ = parser.run().unwrap();
}

#[test]
fn pptr_test() {
    let code = String::from("{int **a;}");
    let mut parser = Parser::load(code);
    parser.init();
    let _ = parser.parse_stmt().unwrap();
    let pptr = parser.local[0][0].clone();
    if let Type::PTR(ptr) = pptr.val_type {
        let ptr_type = *ptr;
        if let Type::PTR(val) = ptr_type {
            assert_eq!(*val, Type::INT);
        } else {
            panic!("expected pointer ");
        } 
    } else {
        panic!("expected pointer to pointer");
    }

    
        
}
