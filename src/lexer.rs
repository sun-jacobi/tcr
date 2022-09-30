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
    Comma,
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
                    ',' => return self.bump(TokenKind::Comma, 1),
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

//------------------------------------------------------------------------

#[cfg(test)]
#[test]
fn add_test() {
    let code = String::from("42 + 31");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("31".to_string()));
}
#[test]
fn three_test() {
    let code = String::from(" 42 + 31 + 18");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("31".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("18".to_string()));
}

#[test]
fn corner_test() {
    let code = String::from(" 42  + 31 +18   ");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("31".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("18".to_string()));
}

#[test]
fn single_test() {
    let code = String::from(" 42");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
}

#[test]
fn muldiv_test() {
    let code = String::from("42 * 31 / 12");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Star);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("31".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Slash);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("12".to_string()));
}

#[test]
fn brackets_test() {
    let code = String::from("(42 * 31 )");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::OpenParen);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Star);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("31".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::CloseParen);
}

#[test]
fn relational_test() {
    let code = String::from("42 >= 31 > 28 == 28");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Geq);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("31".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Gt);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("28".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::DoubleEq);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("28".to_string()));
}

#[test]
fn assign_test() {
    let code = String::from("a=0");
    let mut lexer = Lexer::new(code);
    assert_eq!(
        lexer.next().unwrap().kind,
        TokenKind::Ident("a".to_string())
    );
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Eq);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("0".to_string()));
}
#[test]
fn return_test() {
    let code = String::from("return 42");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Return);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
}

#[test]
fn for_test() {
    let code = String::from("if ( a > 42 )");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::If);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::OpenParen);
    assert_eq!(
        lexer.next().unwrap().kind,
        TokenKind::Ident("a".to_string())
    );
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Gt);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::CloseParen);
}

#[test]
fn block_test() {
    let code = String::from("{42;}");
    let mut lexer = Lexer::new(code);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::OpenCur);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Num("42".to_string()));
    assert_eq!(lexer.next().unwrap().kind, TokenKind::SemiCol);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::CloseCur);
}
