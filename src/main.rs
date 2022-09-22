use std::env::{self, args};

fn main() {
    if env::args().len() != 2 {
        eprintln!("Wrong argument number");
    }
    println!(".intel_syntax noprefix");
    println!(".globl _main");
    println!("_main:");
    println!("  mov rax, {}", args().nth(1).unwrap());
    println!("  ret");
}
