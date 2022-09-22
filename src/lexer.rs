
#[derive(Default)]
pub struct Lexer {
    cursor: usize,
    characters: Vec<char>
}


#[derive(Debug, PartialEq)]
pub enum Token {
    RESERVED(char),
    NUM(i64),
}


impl Iterator for Lexer {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let mut num_literal = String::default();

        loop {
            match self.curr_char() {
                None => {
                    if !num_literal.is_empty(){
                        return Some(Token::NUM(num_literal.parse::<i64>().unwrap()));
                    } 
                    return None;
                }
                Some(c) => match c {
                    ' ' => {
                        if !num_literal.is_empty() {
                            return Some(Token::NUM(num_literal.parse::<i64>().unwrap()));
                        } 
                        self.cursor += 1;
                        continue;
                    },
                    n @ ('+' | '-') => {
                        if !num_literal.is_empty() {
                            return Some(Token::NUM(num_literal.parse::<i64>().unwrap()));
                        }
                        self.cursor += 1;
                        return Some(Token::RESERVED(n));
                    
                    }, 
                    _ => num_literal.push(c),    
                }  
            
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

    fn char_at(&self, index: usize) -> Option<char> {
        if self.cursor >= self.characters.len() {
            None
        } else {
            Some(self.characters[index])
        }    
    }

    fn curr_char(&self) -> Option<char> {
        self.char_at(self.cursor)
    }

    pub fn expect_num(&mut self) -> Result<i64, &str> {
        match self.next() {
            Some(Token::NUM(num)) => Ok(num),
            _ => Err("Error parsing the number"),
        }
    }

}


