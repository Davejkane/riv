use riv::program::Program;

fn main() -> Result<(), String> {
    let mut program = Program::init()?;
    program.run()?;
    Ok(())
}
