use clive::program::Program;

fn main() -> Result<(), String> {
    let mut program = Program::init()?;
    program.run()?;
    Ok(())
}
