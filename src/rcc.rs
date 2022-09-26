use crate::parser::{Node, NodeKind, Parser};
use std::env::args;

pub struct Rcc {
    parser: Parser,
}
impl Rcc {
    pub fn init(src: String) -> Self {
        let parser = Parser::load(src);
        Self { parser }
    }

    fn gen(&mut self, node: Box<Node>) -> Result<(), &'static str> {
        if let NodeKind::NUM(num) = node.kind {
            println!("  push {}", num);
            return Ok(());
        }
        if let Some(lhs) = node.lhs {
            self.gen(lhs)?;
        }
        if let Some(rhs) = node.rhs {
            self.gen(rhs)?;
        }
        println!("  pop rdi");
        println!("  pop rax");
        match node.kind {
            NodeKind::ADD => println!("  add rax, rdi"),
            NodeKind::SUB => println!("  sub rax, rdi"),
            NodeKind::MUL => println!("  imul rax, rdi"),
            NodeKind::DIV => {
                println!("  cqo");
                println!("  idiv rdi");
            }
            NodeKind::Eq => {
                println!("  cmp rax, rdi");
                println!("  sete al");
                println!("  movzx rax, al")
            }
            NodeKind::NotEq => {
                println!("  cmp rax, rdi");
                println!("  setne al");
                println!("  movzx rax, al")
            }
            NodeKind::Lt => {
                println!("  cmp rax, rdi");
                println!("  setle al");
                println!("  movzx rax, al")
            }
            NodeKind::Leq => {
                println!("  cmp rax, rdi");
                println!("  setl al");
                println!("  movzx rax, al")
            }
            _ => return Err("not expected node"),
        }
        println!("  push rax");
        Ok(())
    }

    fn prolog() {
        println!(".intel_syntax noprefix");
        println!(".globl _main");
        println!("_main:");
    }

    fn epilog() {
        println!("  pop rax");
        println!("  ret");
    }

    pub fn run() -> Result<(), &'static str> {
        let src = args().nth(1).expect("Wrong argument number");
        let mut rcc = Rcc::init(src);
        let ast = rcc.parser.run()?;
        Rcc::prolog();
        rcc.gen(ast)?;
        Rcc::epilog();
        Ok(())
    }
}
