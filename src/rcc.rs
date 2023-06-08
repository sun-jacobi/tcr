use crate::parser::{Node, NodeKind, Parser};
use std::env::args;

pub struct Rcc {
    parser: Parser,
    mangle: u8,
}

// C ABI register
const ARG_REGISTER: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

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

    // get a new name-mangling
    fn pop_mangle(&mut self) -> String {
        let mangle = self.mangle;
        self.mangle += 1;
        format!(".L{}", mangle)
    }

    fn gen(&mut self, node: Box<Node>) -> Result<(), String> {
        if let NodeKind::Declar = node.kind {
            let lhs = node.lhs.unwrap();
            if let NodeKind::LVAL(offset) = lhs.kind {
                Self::addr(offset);
                println!("  push 0");
                println!("  pop r10");
                println!("  pop rax");
                println!("  mov [rax], r10");
                println!("  push r10");
                return Ok(());
            } else {
                return Err(String::from("expected lval"));
            }
        }
        // *lval
        if let NodeKind::Deref = node.kind {
            if let Some(rhs) = &node.rhs {
                if let NodeKind::LVAL(offset) = rhs.kind {
                    Self::addr(offset);
                    println!("  pop rax");
                    println!("  mov rax, [rax]");
                    println!("  mov rax, [rax]");
                    println!("  push rax");
                    return Ok(());
                } else {
                    return Err("expected lvalue.".to_string());
                }
            } else {
                return Err("expected expression.".to_string());
            }
        }

        // &lval
        if let NodeKind::Addr = node.kind {
            if let Some(rhs) = &node.rhs {
                if let NodeKind::LVAL(offset) = rhs.kind {
                    Self::addr(offset);
                    return Ok(());
                } else {
                    return Err("expected lvalue.".to_string());
                }
            } else {
                return Err("expected expression.".to_string());
            }
        }

        if let NodeKind::Def {
            name,
            args,
            body,
            local,
        } = node.kind
        {
            println!("_{}:", name);
            let offsets = self.parser.get_local_size(local);
            self.prolog(offsets, args);
            if let NodeKind::Block(stmts) = body.kind {
                for stmt in stmts {
                    self.gen(stmt)?;
                    println!("  pop rax");
                }
            } else {
                return Err("expected function body".to_string());
            }
            Rcc::epilog();
            return Ok(());
        }

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

            println!("  sub rsp, 8"); // Sys-V
            println!("  call _{}", name);
            println!("  add rsp, 8"); // Sys-V
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
                    println!("  je {}", else_mangle);
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
                None => return Err("expected lvalue".to_string()),
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
                    _ => return Err("expected lvalue".to_string()),
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
            _ => return Err("not expected node".to_string()),
        }
        println!("  push rax");
        Ok(())
    }

    fn prefix() {
        println!(".intel_syntax noprefix");
        println!(".globl _main");
        //println!("_main:");
    }

    // rbp : base pointer
    // rsp : stack pointer
    fn prolog(&self, offsets: usize, args: usize) {
        println!("  push rbp");
        println!("  mov rbp, rsp");
        println!("  sub rsp, {}", offsets * 8);
        for arg in 0..args {
            println!("  mov [rbp-{}], {}", (arg + 1) * 8, ARG_REGISTER[arg]);
        }
    }

    fn epilog() {
        println!("  leave");
        println!("  ret");
    }

    pub fn run() -> Result<(), String> {
        let src = args().nth(1).expect("Wrong argument number");
        let mut rcc = Rcc::init(src);
        let program = rcc.parser.run()?;

        Rcc::prefix();
        //Rcc::prolog();

        for func in program {
            rcc.gen(func)?;
        }

        //Rcc::epilog();
        Ok(())
    }
}
