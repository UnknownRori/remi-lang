use remi_lang::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = cli::CLI::new();
    frontend.run()?;
    Ok(())
}
