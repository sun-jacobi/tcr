pub mod lexer;
pub mod parser;

#[cfg(test)]
mod lexer_test {
    use super::lexer::{Lexer, Token};
    #[test]
    fn add_test() {
        let code = String::from("42 + 31");
        let mut lexer = Lexer::new(code);
        assert_eq!(*lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(31));
    }
    #[test]
    fn three_test() {
        let code = String::from(" 42 + 31 + 18");
        let mut lexer = Lexer::new(code);
        assert_eq!(*lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(31));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(18));
    }

    #[test]
    fn corner_test() {
        let code = String::from(" 42  + 31 +18   ");
        let mut lexer = Lexer::new(code);
        assert_eq!(*lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(31));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('+'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(18));
    }

    #[test]
    fn single_test() {
        let code = String::from(" 42");
        let mut lexer = Lexer::new(code);
        assert_eq!(*lexer.next().unwrap(), Token::NUM(42));
    }

    #[test]
    fn muldiv_test() {
        let code = String::from("42 * 31 / 12");
        let mut lexer = Lexer::new(code);
        assert_eq!(*lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('*'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(31));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('/'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(12));
    }

    #[test]
    fn brackets_test() {
        let code = String::from("(42 * 31 )");
        let mut lexer = Lexer::new(code);
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('('));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(42));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED('*'));
        assert_eq!(*lexer.next().unwrap(), Token::NUM(31));
        assert_eq!(*lexer.next().unwrap(), Token::RESERVED(')'));
    }
}

#[cfg(test)]
mod parser_test {
    use super::parser::NodeKind;
    use super::parser::Parser;
    #[test]
    fn add_test() {
        let code = String::from("42 + 31");
        let mut parser = Parser::load(code);
        // root for an ast
        let root = parser.run().unwrap();
        assert_eq!(root.kind, NodeKind::ADD);
        assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(42));
        assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(31));
    }

    #[test]
    fn single_test() {
        let code = String::from("42");
        let mut parser = Parser::load(code);
        // root for an ast
        let root = parser.run().unwrap();
        assert_eq!(root.kind, NodeKind::NUM(42));
    }

    #[test]
    fn mul_test() {
        let code = String::from("42*31");
        let mut parser = Parser::load(code);
        // root for an ast
        let root = parser.run().unwrap();
        assert_eq!(root.kind, NodeKind::MUL);
        assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(42));
        assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(31));
    }



    #[test]
    fn addmul_test() {
        let code = String::from("42 + 31 * 1");
        let mut parser = Parser::load(code);
        // root for an ast
        let root = parser.run().unwrap();
        let lhs  = root.lhs.unwrap();
        let rhs  = root.rhs.unwrap();
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
        // root for an ast
        let root = parser.run().unwrap();
        let lhs  = root.lhs.unwrap();
        let rhs  = root.rhs.unwrap();
        assert_eq!(root.kind, NodeKind::MUL);
        assert_eq!(rhs.kind, NodeKind::ADD);
        assert_eq!(lhs.kind, NodeKind::NUM(42));
        assert_eq!(rhs.lhs.unwrap().kind, NodeKind::NUM(31));
        assert_eq!(rhs.rhs.unwrap().kind, NodeKind::NUM(1));
    }
}
