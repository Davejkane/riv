//! Screen contains the Screen struct which contains all SDL initialised data required
//! for building the window and rendering to screen.
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::{FullscreenType, WindowContext};
use sdl2::Sdl;

/// Screen contains all SDL related data required for running the screen rendering.
pub struct Screen<'a> {
    /// sdl_context is required for running SDL
    pub sdl_context: Sdl,
    /// canvas is where images and text will be rendered
    pub canvas: WindowCanvas,
    /// texture_creator is used for loading images
    pub texture_creator: &'a TextureCreator<WindowContext>,
    /// font is used for printing text
    pub font: Font<'a, 'static>,
    /// mono_font is used for printing mono spaced text
    pub mono_font: Font<'a, 'static>,
    /// last_index is the index of the last texture rendered
    pub last_index: usize,
    /// last_texture is the last image texture rendered
    pub last_texture: Option<sdl2::render::Texture<'a>>,
    /// dirty, if true indicates that last texture must be discarded
    pub dirty: bool,
    /// window title
    pub window_title: String,
    /// x dimension of screen
    pub window_width: u32,
    /// y dimension of screen
    pub window_height: u32,
    /// current screen window is on
    pub current_display: i32,
}

impl Screen<'_> {
    /// Updates window for fullscreen state
    pub fn update_fullscreen(&mut self, fullscreen: bool) -> Result<(), String> {
        use FullscreenType::*;
        let fullscreen_type = if fullscreen { Off } else { True };
        if self.canvas.window().fullscreen_state() == fullscreen_type {
            return Ok(());
        }

        if let Err(e) = self.canvas.window_mut().set_fullscreen(fullscreen_type) {
            return Err(format!("failed to update display: {:?}", e).to_string());
        }
        match fullscreen_type {
            Off => {
                self.sdl_context
                    .video()
                    .unwrap()
                    .window(&self.window_title, self.window_width, self.window_height)
                    .position_centered()
                    .resizable()
                    .build()
                    .unwrap();
            }
            Desktop | True => {
                self.sdl_context
                    .video()
                    .unwrap()
                    .window(&self.window_title, self.window_width, self.window_height)
                    .borderless()
                    .fullscreen()
                    .build()
                    .unwrap();
            }
        };
        Ok(())
    }
}
