mod lexer;
mod parser;
mod rcc;

fn main() {
    rcc::Rcc::default().run();
}
