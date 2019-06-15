//! Screen contains the Screen struct which contains all SDL initialised data required
//! for building the window and rendering to screen.
use crate::infobar::Text;
use crate::program::{Colors, HALF_PAD, PADDING};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::{FullscreenType, WindowContext};
use sdl2::Sdl;
use FullscreenType::*;

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
    pub last_index: Option<usize>,
    /// last_texture is the last image texture rendered
    pub last_texture: Option<sdl2::render::Texture<'a>>,
    /// dirty, if true indicates that last texture must be discarded
    pub dirty: bool,
}

impl Screen<'_> {
    /// Renders infobar on screen
    pub fn render_infobar(
        &mut self,
        text: Text,
        text_color: Color,
        theme: &Colors,
    ) -> Result<(), String> {
        let text = text; //infobar::Text::update(&self.ui_state.mode, &self.paths, &self.ui_state);
                         // Load the filename texture
        let filename_surface = self
            .font
            .render(&text.child_2)
            .blended(text_color)
            .map_err(|e| e.to_string())?;
        let filename_texture = self
            .texture_creator
            .create_texture_from_surface(&filename_surface)
            .map_err(|e| e.to_string())?;
        let filename_dimensions = filename_texture.query();
        // Load the index texture
        let index_surface = self
            .font
            .render(&text.child_1)
            .blended(text_color)
            .map_err(|e| e.to_string())?;
        let index_texture = self
            .texture_creator
            .create_texture_from_surface(&index_surface)
            .map_err(|e| e.to_string())?;
        let index_dimensions = index_texture.query();
        // Draw the Bar
        let dims = (
            index_dimensions.height,
            index_dimensions.width,
            filename_dimensions.width,
        );
        self.render_bar(dims, theme)?;
        // Copy the text textures
        let y = (self.canvas.viewport().height() - index_dimensions.height) as i32;
        if let Err(e) = self.canvas.copy(
            &index_texture,
            None,
            Rect::new(
                PADDING as i32,
                y,
                index_dimensions.width,
                index_dimensions.height,
            ),
        ) {
            eprintln!("Failed to copy text to screen {}", e);
        }
        if let Err(e) = self.canvas.copy(
            &filename_texture,
            None,
            Rect::new(
                (index_dimensions.width + PADDING as u32 * 2) as i32,
                y,
                filename_dimensions.width,
                filename_dimensions.height,
            ),
        ) {
            eprintln!("Failed to copy text to screen {}", e);
            return Ok(());
        }
        Ok(())
    }

    fn render_bar(&mut self, dims: (u32, u32, u32), theme: &Colors) -> Result<(), String> {
        let colors = theme;
        let height = dims.0;
        let width = self.canvas.viewport().width();
        let y = (self.canvas.viewport().height() - height) as i32;
        let mut x = 0;
        let mut w = dims.1 + HALF_PAD as u32 * 3;
        self.canvas.set_draw_color(colors.bg_rect_left);
        if let Err(e) = self.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = dims.2 + PADDING as u32 * 2;
        self.canvas.set_draw_color(colors.bg_rect_2);
        if let Err(e) = self.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = width;
        self.canvas.set_draw_color(colors.bg_rest);
        if let Err(e) = self.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        Ok(())
    }

    /// Updates window for fullscreen state
    pub fn update_fullscreen(&mut self, fullscreen: bool) -> Result<(), String> {
        let fullscreen_type = if fullscreen { Off } else { True };
        if self.canvas.window().fullscreen_state() == fullscreen_type {
            return Ok(());
        }

        if let Err(e) = self.canvas.window_mut().set_fullscreen(fullscreen_type) {
            return Err(format!("failed to update display: {:?}", e).to_string());
        }
        match fullscreen_type {
            Off => {
                let window = self.canvas.window_mut();
                window.set_fullscreen(Off).unwrap();
                window.set_bordered(true);
            }
            Desktop | True => {
                let window = self.canvas.window_mut();
                window.set_bordered(false);
                window.set_fullscreen(True).unwrap();
            }
        };
        Ok(())
    }
}
