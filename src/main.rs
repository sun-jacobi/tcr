mod lexer;
mod parser;
mod rcc;

fn main() -> Result<(), &'static str> {
    rcc::Rcc::run()?;
    Ok(())
}
