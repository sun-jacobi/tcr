pub mod lexer;

#[cfg(test)]
mod tests {
    use super::lexer::{Lexer, Token};
    #[test]
    fn add_test() {
        let code = String::from("42 + 31");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(lexer.next().unwrap(), Token::NUM(31));     
    }
    #[test]
    fn three_test() {
        let code = String::from(" 42 + 31 + 18");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(lexer.next().unwrap(), Token::NUM(31)); 
        assert_eq!(lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(lexer.next().unwrap(), Token::NUM(18)); 
    }

    #[test]
    fn corner_test() {
        let code = String::from(" 42  + 31 +18   ");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(lexer.next().unwrap(), Token::NUM(31)); 
        assert_eq!(lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(lexer.next().unwrap(), Token::NUM(18)); 
    }

    #[test]
    fn single_test() {
        let code = String::from(" 42");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap(), Token::NUM(42));
    }
}
