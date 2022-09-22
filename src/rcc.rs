use std::env::args;

use crate::lexer::{Lexer, Token}; 


#[derive(Default)]
pub struct Rcc {
    //TODO
}
impl Rcc {
    pub fn run(&mut self) {
        println!(".intel_syntax noprefix");
        println!(".globl _main");
        println!("_main:");
        let src = args().nth(1).expect("Wrong argument number");
        let mut lexer = Lexer::new(src);
        println!("  mov rax, {}", lexer.expect_num().unwrap());
        loop {
            match lexer.next() {
                None => break,
                Some(t) => match t {
                    Token::RESERVED('+') => println!("  add rax, {}", 
                                        lexer.expect_num().unwrap()),
                    Token::RESERVED('-') => println!("  sub rax, {}", 
                                        lexer.expect_num().unwrap()),
                    _ => panic!("Error"),
                }
                
            }
        }
        println!("  ret");
    }
}
