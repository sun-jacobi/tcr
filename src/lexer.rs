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
    NotEq,
    DoubleEq,
    Eq,
    OpenParen,  // (
    CloseParen, // )
    OpenCur,    // {
    CloseCur,   // }
    SemiCol,
    Return,
    If,
    For,
    While,
    Else,
    Num(String),
    Ident(String),
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

    pub(crate) fn num(state: Vec<char>) -> Option<Box<Self>> {
        let len = state.len();
        Some(Box::new(Self::new(
            TokenKind::Num(state.into_iter().collect()),
            len,
        )))
    }

    pub(crate) fn word(state: Vec<char>) -> Option<Box<Self>> {
        let len = state.len();
        let word = state.into_iter().collect::<String>();
        match word.as_str() {
            "else" => Some(Box::new(Self::new(TokenKind::Else, len))),
            "if" => Some(Box::new(Self::new(TokenKind::If, len))),
            "while" => Some(Box::new(Self::new(TokenKind::While, len))),
            "for" => Some(Box::new(Self::new(TokenKind::For, len))),
            "return" => Some(Box::new(Self::new(TokenKind::Return, len))),
            _ => Some(Box::new(Self::new(TokenKind::Ident(word), len))),
        }
    }
}

impl Iterator for Lexer {
    type Item = Box<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.first() {
                None => return None,
                Some(&c) => match c {
                    '0'..='9' => return self.num(),
                    'a'..='z' | 'A'..='Z' => return self.word(),
                    '{' => return self.bump(TokenKind::OpenCur, 1),
                    '}' => return self.bump(TokenKind::CloseCur, 1),
                    '(' => return self.bump(TokenKind::OpenParen, 1),
                    ')' => return self.bump(TokenKind::CloseParen, 1),
                    '+' => return self.bump(TokenKind::Add, 1),
                    '-' => return self.bump(TokenKind::Minus, 1),
                    '*' => return self.bump(TokenKind::Star, 1),
                    '/' => return self.bump(TokenKind::Slash, 1),
                    ';' => return self.bump(TokenKind::SemiCol, 1),
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
                    '!' => {
                        if let Some('=') = self.second() {
                            return self.bump(TokenKind::NotEq, 2);
                        }
                        panic!("Expected `=`");
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

    fn num(&mut self) -> Option<Box<Token>> {
        let mut state: Vec<char> = Vec::new();
        loop {
            match self.first() {
                None => return Token::num(state),
                Some(&c) => match c {
                    '0'..='9' => {
                        state.push(c);
                        self.cursor += 1;
                    }
                    _ => return Token::num(state),
                },
            }
        }
    }
    fn word(&mut self) -> Option<Box<Token>> {
        let mut state: Vec<char> = Vec::new();
        loop {
            match self.first() {
                None => return Token::word(state),
                Some(&c) => match c {
                    'a'..='z' | 'A'..='Z' => {
                        state.push(c);
                        self.cursor += 1;
                    }
                    _ => return Token::word(state),
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
