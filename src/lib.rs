pub mod lexer;
pub mod parser;

#[cfg(test)]
mod lexer_test {
    use super::lexer::Lexer;
    use crate::lexer::TokenKind;
    #[test]
    fn add_test() {
        let code = String::from("42 + 31");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("31".to_string()));
    }
    #[test]
    fn three_test() {
        let code = String::from(" 42 + 31 + 18");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("31".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("18".to_string()));
    }

    #[test]
    fn corner_test() {
        let code = String::from(" 42  + 31 +18   ");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("31".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Add);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("18".to_string()));
    }

    #[test]
    fn single_test() {
        let code = String::from(" 42");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
    }

    #[test]
    fn muldiv_test() {
        let code = String::from("42 * 31 / 12");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Star);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("31".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Slash);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("12".to_string()));
    }

    #[test]
    fn brackets_test() {
        let code = String::from("(42 * 31 )");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::OpenParen);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Star);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("31".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::CloseParen);
    }

    #[test]
    fn relational_test() {
        let code = String::from("42 >= 31 > 28 == 28");
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("42".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Geq);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("31".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Gt);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("28".to_string()));
        assert_eq!(lexer.next().unwrap().kind, TokenKind::DoubleEq);
        assert_eq!(lexer.next().unwrap().kind, TokenKind::Lit("28".to_string()));
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
        // root for an ast
        let root = parser.run().unwrap();
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
        // root for an ast
        let root = parser.run().unwrap();
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
        let root = parser.run().unwrap();
        assert_eq!(root.kind, NodeKind::Leq);
        assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(42));
        assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(31));
    }

    #[test]
    fn two_relation_test() {
        let code = String::from("42 * 31 >=   31 + 42");
        let mut parser = Parser::load(code);
        let root = parser.run().unwrap();
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
}
