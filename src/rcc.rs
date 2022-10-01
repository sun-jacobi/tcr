use crate::parser::{Node, NodeKind, Parser};
use std::env::args;

pub struct Rcc {
    parser: Parser,
    mangle: u8,
}

const ARG_REGISTER: [&'static str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

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

    fn pop_mangle(&mut self) -> String {
        let mangle = self.mangle;
        self.mangle += 1;
        format!(".L{}", mangle)
    }

    fn gen(&mut self, node: Box<Node>) -> Result<(), &'static str> {
        if let NodeKind::Nop = node.kind {
            println!("  nop");
            return Ok(());
        }

        if let NodeKind::Func { name, argv } = node.kind {
            for (index, arg) in argv.into_iter().enumerate() {
                self.gen(arg)?;
                println!("  pop rax");
                println!("  mov {}, rax", ARG_REGISTER[index]);
            }

            Rcc::align();
            println!("  call _{}", name);
            println!("  push rax");
            return Ok(());
        }

        if let NodeKind::Block(stmts) = node.kind {
            for stmt in stmts {
                self.gen(stmt)?;
                println!("  pop rax");
            }
            return Ok(());
        }

        if let NodeKind::For { init, end, inc } = node.kind {
            let stmt = node.lhs.unwrap();
            self.gen(init)?;
            let condition_mangle = self.pop_mangle();
            let end_mangle = self.pop_mangle();
            println!("{}:", condition_mangle);
            self.gen(end)?;
            println!("  pop rax");
            println!("  cmp rax, 0"); // if A = 0
            println!("  je {}", end_mangle);
            self.gen(stmt)?;
            self.gen(inc)?;
            println!("  jmp {}", condition_mangle);
            println!("{}:", end_mangle);
            return Ok(());
        }

        if let NodeKind::While = node.kind {
            let condition = node.lhs.unwrap();
            let stmt = node.rhs.unwrap();
            let condtion_mangle = self.pop_mangle();
            println!("{}:", condtion_mangle);
            self.gen(condition)?;
            println!("  pop rax");
            println!("  cmp rax, 0"); // if A = 0
            let end_mangle = self.pop_mangle();
            println!("  je {}", end_mangle);
            self.gen(stmt)?;
            println!("  jmp {}", condtion_mangle);
            println!("{}:", end_mangle);
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
                    println!(" je {}", else_mangle);
                    self.gen(node.lhs.unwrap())?;
                    let end_mangle = self.pop_mangle();
                    println!("  jmp {}", end_mangle);
                    println!("{}:", else_mangle);
                    self.gen(rhs)?;
                    println!("{}:", end_mangle);
                    return Ok(());
                }
                // if
                None => {
                    let end_mangle = self.pop_mangle();
                    println!("  je {}", end_mangle);
                    self.gen(node.lhs.unwrap())?;
                    println!("{}:", end_mangle);
                    return Ok(());
                }
            }
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
                        println!("  pop r10");
                        println!("  pop rax");
                        println!("  mov [rax], r10");
                        println!("  push r10");
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
            println!("  pop rax");
            Rcc::epilog();
            return Ok(());
        }

        println!("  pop r10");
        println!("  pop rax");
        match node.kind {
            NodeKind::ADD => println!("  add rax, r10"),
            NodeKind::SUB => println!("  sub rax, r10"),
            NodeKind::MUL => println!("  imul rax, r10"),
            NodeKind::DIV => {
                println!("  cqo");
                println!("  idiv r10");
            }
            NodeKind::Eq => {
                println!("  cmp rax, r10");
                println!("  sete al");
                println!("  movzx rax, al")
            }
            NodeKind::NotEq => {
                println!("  cmp rax, r10");
                println!("  setne al");
                println!("  movzx rax, al")
            }
            NodeKind::Lt => {
                println!("  cmp rax, r10");
                println!("  setl al");
                println!("  movzx rax, al")
            }
            NodeKind::Leq => {
                println!("  cmp rax, r10");
                println!("  setle al");
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

    // 16 bytes alignment
    fn align() {
        println!("  shr rsp, 4");
        println!("  sub rsp, 1");
        println!("  shl rsp, 4");
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
            rcc.gen(stmt)?;
            println!("  pop rax");
        }

        Rcc::epilog();
        Ok(())
    }
}
