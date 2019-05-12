use crate::infobar;
use crate::program::{make_dst, Program};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

const PADDING: i32 = 30;

impl<'a> Program<'a> {
    /// render loads the image at the path in the images path vector located at the index and
    /// renders to screen
    pub fn render(&mut self) -> Result<(), String> {
        self.screen.canvas.set_draw_color(dark_grey());
        if self.paths.images.is_empty() {
            return self.render_blank();
        }
        self.screen.canvas.clear();
        self.render_image()?;
        if self.ui_state.render_infobar {
            self.render_infobar()?;
        }

        // Present to screen
        self.screen.canvas.present();
        Ok(())
    }

    fn render_image(&mut self) -> Result<(), String> {
        self.set_image_texture()?;
        match self.screen.last_texture {
            Some(_) => (),
            None => return Ok(()),
        };
        let tex = self.screen.last_texture.as_ref().unwrap();
        let query = tex.query();
        let target = self.screen.canvas.viewport();
        let dest = make_dst(query.width, query.height, target.width(), target.height());
        if let Err(e) = self.screen.canvas.copy(tex, None, dest) {
            eprintln!("Failed to copy image to screen {}", e);
        }
        Ok(())
    }

    fn set_image_texture(&mut self) -> Result<(), String> {
        if self.paths.index == self.screen.last_index
            && self.screen.last_texture.is_some()
            && !self.screen.dirty
        {
            return Ok(());
        }
        let texture = match self
            .screen
            .texture_creator
            .load_texture(&self.paths.images[self.paths.index])
        {
            Ok(t) => {
                self.screen.last_index = self.paths.index;
                t
            }
            Err(e) => {
                eprintln!("Failed to render image {}", e);
                return Ok(());
            }
        };
        self.screen.last_texture = Some(texture);
        self.screen.dirty = false;
        Ok(())
    }

    fn render_infobar(&mut self) -> Result<(), String> {
        let text = infobar::Text::from(&self.paths);
        // Load the filename texture
        let filename_surface = self
            .screen
            .font
            .render(&text.current_image)
            .blended(text_color())
            .map_err(|e| e.to_string())?;
        let filename_texture = self
            .screen
            .texture_creator
            .create_texture_from_surface(&filename_surface)
            .map_err(|e| e.to_string())?;
        let filename_dimensions = filename_texture.query();
        // Load the index texture
        let index_surface = self
            .screen
            .font
            .render(&text.index)
            .blended(text_color())
            .map_err(|e| e.to_string())?;
        let index_texture = self
            .screen
            .texture_creator
            .create_texture_from_surface(&index_surface)
            .map_err(|e| e.to_string())?;
        let index_dimensions = index_texture.query();
        // Load the context texture
        let context_surface = self
            .screen
            .font
            .render(&text.context)
            .blended(text_color())
            .map_err(|e| e.to_string())?;
        let context_texture = self
            .screen
            .texture_creator
            .create_texture_from_surface(&context_surface)
            .map_err(|e| e.to_string())?;
        let context_dimensions = context_texture.query();
        // Draw the Bar
        let dims = (
            index_dimensions.height,
            context_dimensions.width,
            index_dimensions.width,
            filename_dimensions.width,
        );
        self.render_bar(dims)?;
        // Copy the text textures
        let y = (self.screen.canvas.viewport().height() - index_dimensions.height) as i32;
        if let Err(e) = self.screen.canvas.copy(
            &context_texture,
            None,
            Rect::new(
                PADDING,
                y,
                context_dimensions.width,
                context_dimensions.height,
            ),
        ) {
            eprintln!("Failed to copy text to screen {}", e);
        }
        if let Err(e) = self.screen.canvas.copy(
            &index_texture,
            None,
            Rect::new(
                (context_dimensions.width + PADDING as u32 * 2) as i32,
                y,
                index_dimensions.width,
                index_dimensions.height,
            ),
        ) {
            eprintln!("Failed to copy text to screen {}", e);
        }
        if let Err(e) = self.screen.canvas.copy(
            &filename_texture,
            None,
            Rect::new(
                (context_dimensions.width + index_dimensions.width + PADDING as u32 * 3) as i32,
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

    fn render_bar(&mut self, dims: (u32, u32, u32, u32)) -> Result<(), String> {
        let height = dims.0;
        let padding = 15;
        let width = self.screen.canvas.viewport().width();
        let y = (self.screen.canvas.viewport().height() - height) as i32;
        let mut x = 0;
        let mut w = dims.1 + padding * 3;
        self.screen.canvas.set_draw_color(red());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = dims.2 + padding * 2;
        self.screen.canvas.set_draw_color(orange());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = dims.3 + padding * 2;
        self.screen.canvas.set_draw_color(green());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = width;
        self.screen.canvas.set_draw_color(light_grey());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        Ok(())
    }

    fn render_blank(&mut self) -> Result<(), String> {
        self.screen.canvas.clear();
        if self.ui_state.render_infobar {
            self.render_infobar()?;
        }
        self.screen.canvas.present();
        Ok(())
    }
}

fn dark_grey() -> Color {
    Color::RGB(45, 45, 45)
}

fn red() -> Color {
    Color::RGB(255, 69, 58)
}

fn orange() -> Color {
    Color::RGB(255, 159, 10)
}

fn green() -> Color {
    Color::RGB(50, 215, 75)
}

fn light_grey() -> Color {
    Color::RGB(152, 152, 157)
}

fn text_color() -> Color {
    Color::RGBA(255, 228, 204, 255)
}
