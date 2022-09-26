pub(crate) struct Lexer {
    cursor: usize,
    characters: Vec<char>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TokenKind {
    Add,
    Minus,
    Star,
    Slash,
    Lt,
    Gt,
    Geq,
    Leq,
    DoubleEq,
    Eq,
    OpenParen,
    CloseParen,
    Lit(String),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) len: usize,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, len: usize) -> Self {
        Self { kind, len }
    }
    pub(crate) fn sym(kind: TokenKind, len: usize) -> Option<Box<Self>> {
        Some(Box::new(Self::new(kind, len)))
    }
    pub(crate) fn lit(state: Vec<char>) -> Option<Box<Self>> {
        let len = state.len();
        Some(Box::new(Self::new(
            TokenKind::Lit(state.into_iter().collect()),
            len,
        )))
    }
}

impl Iterator for Lexer {
    type Item = Box<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.first() {
                None => return None,
                Some(&c) => match c {
                    '1'..='9' => return self.eat(),
                    '(' => return self.bump(TokenKind::OpenParen, 1),
                    ')' => return self.bump(TokenKind::CloseParen, 1),
                    '+' => return self.bump(TokenKind::Add, 1),
                    '-' => return self.bump(TokenKind::Minus, 1),
                    '*' => return self.bump(TokenKind::Star, 1),
                    '/' => return self.bump(TokenKind::Slash, 1),
                    '>' => {
                        if let Some('=') = self.second() {
                            return self.bump(TokenKind::Geq, 2);
                        }
                        return self.bump(TokenKind::Gt, 1);
                    }
                    '<' => {
                        if let Some('=') = self.second() {
                            return self.bump(TokenKind::Leq, 2);
                        }
                        return self.bump(TokenKind::Lt, 1);
                    }
                    '=' => {
                        if let Some('=') = self.second() {
                            return self.bump(TokenKind::DoubleEq, 2);
                        }
                        return self.bump(TokenKind::Eq, 1);
                    }
                    _ => self.advance(),
                },
            }
        }
    }
}

impl Lexer {
    pub(crate) fn new(src: String) -> Self {
        Self {
            cursor: 0,
            characters: src.chars().collect(),
        }
    }

    fn eat(&mut self) -> Option<Box<Token>> {
        let mut state: Vec<char> = Vec::new();
        loop {
            match self.first() {
                None => return Token::lit(state),
                Some(&c) => match c {
                    '0'..='9' => {
                        state.push(c);
                        self.cursor += 1;
                    }
                    _ => return Token::lit(state),
                },
            }
        }
    }
    fn bump(&mut self, kind: TokenKind, len: usize) -> Option<Box<Token>> {
        self.cursor += len;
        Token::sym(kind, len)
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }
    fn second(&mut self) -> Option<&char> {
        self.characters.get(self.cursor + 1)
    }

    fn first(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }
}
