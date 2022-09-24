#[derive(Default)]
pub struct Lexer {
    cursor: usize,
    characters: Vec<char>,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    RESERVED(char),
    NUM(i64),
}

impl Iterator for Lexer {
    type Item = Box<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut num_literal = String::default();
        loop {
            match self.curr_char() {
                None => {
                    if !num_literal.is_empty() {
                        return Some(Box::new(Token::NUM(num_literal.parse::<i64>().unwrap())));
                    }
                    return None;
                }
                Some(&c) => match c {
                    ' ' => {
                        if !num_literal.is_empty() {
                            return Some(Box::new(Token::NUM(num_literal.parse::<i64>().unwrap())));
                        }
                        self.cursor += 1;
                        continue;
                    }
                    n @ ('+' | '-' | '*' | '/' | '(' | ')') => {
                        if !num_literal.is_empty() {
                            return Some(Box::new(Token::NUM(num_literal.parse::<i64>().unwrap())));
                        }
                        self.cursor += 1;
                        return Some(Box::new(Token::RESERVED(n)));
                    }
                    _ => num_literal.push(c),
                },
            }
            self.cursor += 1;
        }
    }
}

impl Lexer {
    pub fn new(src: String) -> Self {
        Self {
            cursor: 0,
            characters: src.chars().collect(),
        }
    }

    fn curr_char(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    pub fn expect_num(&mut self) -> Result<i64, &'static str> {
        match self.next() {
            Some(token) => match *token {
                Token::NUM(num) => Ok(num),
                _ => Err("Not Number"),
            },
            None => Err("No new token"),
        }
    }

    pub fn expect(&mut self, expected: Token) -> Result<Box<Token>, &'static str> {
        match self.next() {
            Some(actual) => {
                if *actual == expected {
                    Ok(actual)
                } else {
                    Err("No such Token")
                }
            }
            None => Err("No new token"),
        }
    }
    
}
