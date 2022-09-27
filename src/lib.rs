pub mod lexer;
pub mod parser;

#[cfg(test)]
mod lexer_test {
    use super::lexer::Lexer;
    use super::lexer::TokenKind;
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
}

#[cfg(test)]
mod parser_test {
    use crate::parser::NodeKind;
    use crate::parser::Parser;
    #[test]
    fn add_test() {
        let code = String::from("42 + 31");
        let mut parser = Parser::load(code);
        parser.init();
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
        parser.init();
        // root for an ast
        let root = parser.parse_expr().unwrap();
        assert_eq!(root.kind, NodeKind::NUM(42));
    }

    #[test]
    fn mul_test() {
        let code = String::from("42*31");
        let mut parser = Parser::load(code);
        parser.init();
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
        parser.init();
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
        parser.init();
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
        parser.init();
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
        parser.init();
        let root = parser.parse_expr().unwrap();
        assert_eq!(root.kind, NodeKind::Leq);
        assert_eq!(root.rhs.unwrap().kind, NodeKind::NUM(42));
        assert_eq!(root.lhs.unwrap().kind, NodeKind::NUM(31));
    }

    #[test]
    fn two_relation_test() {
        let code = String::from("42 * 31 >=   31 + 42");
        let mut parser = Parser::load(code);
        parser.init();
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
    fn program_test() {
        let code = "a = 42; b = 31;".to_string();
        let mut parser = Parser::load(code);
        let stmts = parser.run().unwrap();
        assert_eq!(stmts.len(), 2);
        let first = stmts[0].clone();
        let second = stmts[1].clone();
        assert_eq!(first.kind, NodeKind::Assign);
        assert_eq!(first.lhs.unwrap().kind, NodeKind::LVAL(8));
        assert_eq!(first.rhs.unwrap().kind, NodeKind::NUM(42));
        assert_eq!(second.kind, NodeKind::Assign);
        assert_eq!(second.lhs.unwrap().kind, NodeKind::LVAL(16));
        assert_eq!(second.rhs.unwrap().kind, NodeKind::NUM(31));
    }

    #[test]
    fn assign_test() {
        let code = "a = 42; b = 31; a = 31;".to_string();
        let mut parser = Parser::load(code);
        let stmts = parser.run().unwrap();
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
    }
}
