use crate::program::{Program, make_dst};
use crate::infobar;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

impl<'a> Program<'a> {
    /// render loads the image at the path in the images path vector located at the index and
    /// renders to screen
    pub fn render(&mut self) -> Result<(), String> {
        self.screen.canvas.set_draw_color(Color::RGB(45, 45, 45));
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
        if self.paths.index == self.screen.last_index && !self.screen.last_texture.is_none() {
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
        Ok(())
    }

    fn render_infobar(&mut self) -> Result<(), String> {
        let text = infobar::Text::from(&self.paths);
        // Load the filename texture
        let filename_surface = self
            .screen
            .font
            .render(&text.current_image)
            .blended(Color::RGBA(224, 228, 204, 255))
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
            .blended(Color::RGBA(255, 228, 204, 255))
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
            .blended(Color::RGBA(255, 228, 204, 255))
            .map_err(|e| e.to_string())?;
        let context_texture = self
            .screen
            .texture_creator
            .create_texture_from_surface(&context_surface)
            .map_err(|e| e.to_string())?;
        let context_dimensions = context_texture.query();
        // Draw the Bar
        self.screen.canvas.set_draw_color(Color::RGB(243, 134, 48));
        let height = filename_dimensions.height;
        let width = self.screen.canvas.viewport().width();
        let x = 0;
        let y = (self.screen.canvas.viewport().height() - height) as i32;
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, width, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        // Copy the text textures
        if let Err(e) = self.screen.canvas.copy(
            &context_texture,
            None,
            Rect::new(30, y, context_dimensions.width, context_dimensions.height),
        ) {
            eprintln!("Failed to copy text to screen {}", e);
        }
        if let Err(e) = self.screen.canvas.copy(
            &index_texture,
            None,
            Rect::new(
                (context_dimensions.width + 60) as i32,
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
                (context_dimensions.width + index_dimensions.width + 90) as i32,
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

    fn render_blank(&mut self) -> Result<(), String> {
        self.screen.canvas.clear();
        if self.ui_state.render_infobar {
            self.render_infobar()?;
        }
        self.screen.canvas.present();
        Ok(())
    }
}
