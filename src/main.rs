use riv::cli::cli;
use riv::program::Program;
use std::convert::TryInto;

const INITIAL_TITLE: &str = "riv";

fn main() -> Result<(), String> {
    let args = cli()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;

    // Use current display bounds for initial creation
    // Display program was launched on is number 0.
    let display_mode = video.current_display_mode(0).unwrap();
    // Don't see how width or height of a display could be negative
    let program_width = display_mode.w.try_into().unwrap();
    let program_height = display_mode.h.try_into().unwrap();

    let mut window_builder = video.window(INITIAL_TITLE, program_width, program_height);
    if args.fullscreen {
        window_builder.fullscreen().borderless();
    } else {
        window_builder.position_centered();
    }
    // Still keep resizable windows for toggling between fullscreen at runtime
    let window = window_builder
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let mut program = Program::init(&ttf_context, sdl_context, canvas, &texture_creator, args)?;
    program.run()?;
    Ok(())
}
