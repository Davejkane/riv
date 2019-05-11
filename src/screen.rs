//! Screen contains the Screen struct which contains all SDL initialised data required
//! for building the window and rendering to screen.

use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
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
    /// last_index is the index of the last texture rendered
    pub last_index: usize,
    /// last_texture is the last image texture rendered
    pub last_texture: Option<sdl2::render::Texture<'a>>,
}
