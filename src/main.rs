mod lexer;
mod parser;
mod rcc;

fn main() -> Result<(), String> {
    rcc::Rcc::run()?;
    Ok(())
}
