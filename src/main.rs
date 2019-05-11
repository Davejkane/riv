use riv::program::Program;

fn main() -> Result<(), String> {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;
    let window = video
        .window(
            "rust-sdl2 demo: Video",
            video.display_bounds(0).unwrap().width(),
            video.display_bounds(0).unwrap().height(),
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut program = Program::init(&ttf_context, sdl_context, canvas, &texture_creator)?;
    program.run()?;
    Ok(())
}
