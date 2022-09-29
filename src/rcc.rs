use crate::parser::{Node, NodeKind, Parser};
use std::env::args;

pub struct Rcc {
    parser: Parser,
    mangle: u8,
}
impl Rcc {
    pub fn init(src: String) -> Self {
        let parser = Parser::load(src);
        Self { parser, mangle: 1 }
    }

    // push the variable address into the stack
    fn addr(offset: u8) {
        println!("  mov rax, rbp");
        println!("  sub rax, {}", offset);
        println!("  push rax");
    }

    fn peek_mangle(&self) -> u8 {
        self.mangle
    }
    fn pop_mangle(&mut self) -> u8 {
        let mangle = self.mangle;
        self.mangle += 1;
        return mangle;
    }

    fn gen(&mut self, node: Box<Node>) -> Result<(), &'static str> {
        if let NodeKind::Nop = node.kind {
            println!("  nop");
            return Ok(());
        }

        if let NodeKind::If(condition) = node.kind {
            self.gen(condition)?;
            println!("  pop rax");
            println!("  cmp rax, 0"); // if A = 0
            match node.rhs {
                // if-else
                Some(rhs) => {
                    let else_mangle = self.pop_mangle();
                    println!(" je .L{}", else_mangle);
                    self.gen(node.lhs.unwrap())?;
                    println!("  jmp .L{}", self.peek_mangle());
                    println!(".L{}:", else_mangle);
                    self.gen(rhs)?;
                }
                // if
                None => {
                    println!("  je .L{}", self.peek_mangle());
                    self.gen(node.lhs.unwrap())?;
                }
            }
            println!(".L{}:", self.pop_mangle());
            return Ok(());
        }
        if let NodeKind::NUM(num) = node.kind {
            println!("  push {}", num);
            return Ok(());
        }

        // get the value of variable
        if let NodeKind::LVAL(offset) = node.kind {
            Self::addr(offset);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return Ok(());
        }
        // assign the right value to lvalue
        if let NodeKind::Assign = node.kind {
            match &node.lhs {
                None => return Err("expected lvalue"),
                Some(lhs) => match lhs.kind {
                    NodeKind::LVAL(offset) => {
                        Self::addr(offset);
                        self.gen(node.rhs.unwrap())?;
                        println!("  pop rdi");
                        println!("  pop rax");
                        println!("  mov [rax], rdi");
                        println!("  push rdi");
                        return Ok(());
                    }
                    _ => return Err("expected lvalue"),
                },
            }
        }

        if let Some(lhs) = node.lhs {
            self.gen(lhs)?;
        }

        if let Some(rhs) = node.rhs {
            self.gen(rhs)?;
        }
        if let NodeKind::Return = node.kind {
            return Ok(());
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

    fn prefix() {
        println!(".intel_syntax noprefix");
        println!(".globl _main");
        println!("_main:");
    }

    // rbp : base pointer
    // rsp : stack pointer
    fn prolog() {
        println!("  push rbp");
        println!("  mov rbp, rsp");
        println!("  sub rsp, 208");
    }

    fn epilog() {
        println!("  mov rsp, rbp");
        println!("  pop rbp");
        println!("  ret");
    }

    pub fn run() -> Result<(), &'static str> {
        let src = args().nth(1).expect("Wrong argument number");
        let mut rcc = Rcc::init(src);
        let program = rcc.parser.run()?;

        Rcc::prefix();
        Rcc::prolog();

        for stmt in program {
            if let NodeKind::Return = stmt.kind {
                rcc.gen(stmt)?;
                println!("  pop rax");
                break;
            }
            rcc.gen(stmt)?;
            println!("  pop rax");
        }

        Rcc::epilog();
        Ok(())
    }
}
