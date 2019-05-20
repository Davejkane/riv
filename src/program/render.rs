use crate::infobar;
use crate::program::{compute_center_rectangle_view, make_dst, Program};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;

const PADDING: i32 = 30;
const HALF_PAD: i32 = 15;
const LINE_HEIGHT: i32 = 22;
const LINE_PADDING: i32 = 5;

impl<'a> Program<'a> {
    /// render_screen is the main render function that delegates rendering every thing that needs be
    /// rendered
    pub fn render_screen(&mut self, force_render: bool) -> Result<(), String> {
        self.screen.canvas.set_draw_color(dark_grey());
        if self.paths.images.is_empty() {
            return self.render_blank();
        }
        self.screen.canvas.clear();
        self.render_image(force_render)?;
        if self.ui_state.render_infobar {
            self.render_infobar()?;
        }
        if self.ui_state.render_help {
            self.render_help()?;
        }

        // Present to screen
        self.screen.canvas.present();
        Ok(())
    }

    fn render_image(&mut self, force_render: bool) -> Result<(), String> {
        self.set_image_texture(force_render)?;
        match self.screen.last_texture {
            Some(_) => (),
            None => return Ok(()),
        };
        let tex = self.screen.last_texture.as_ref().unwrap();
        let query = tex.query();
        // Area to render other rectangle on
        let target = self.screen.canvas.viewport();
        if self.ui_state.actual_size {
            // Get slice of texture to place on screen
            let content_slice = compute_center_rectangle_view(query.width, query.height, &target);
            let dest = make_dst(
                content_slice.width(),
                content_slice.height(),
                target.width(),
                target.height(),
            );
            if let Err(e) = self.screen.canvas.copy(&tex, content_slice, dest) {
                eprintln!("Failed to copy image to screen {}", e);
            }
        } else {
            let dest = make_dst(query.width, query.height, target.width(), target.height());
            if let Err(e) = self.screen.canvas.copy(&tex, None, dest) {
                eprintln!("Failed to copy image to screen {}", e);
            }
        }
        Ok(())
    }

    fn set_image_texture(&mut self, force_render: bool) -> Result<(), String> {
        if self.paths.index == self.screen.last_index
            && self.screen.last_texture.is_some()
            && !self.screen.dirty
            && !force_render
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

        // Set the default state for viewing of the image
        let query = texture.query();
        let src = Rect::new(0, 0, query.width, query.height);
        let dest = self.screen.canvas.viewport();
        self.ui_state.actual_size = Program::default_actual_size(&src, &dest);

        self.screen.last_texture = Some(texture);
        self.screen.dirty = false;
        Ok(())
    }

    /// Computes the default state of actual_size for each image
    pub fn default_actual_size(src_dims: &Rect, dest_dims: &Rect) -> bool {
        // If any dimension of the src image is bigger than the destination
        // dimensions, use scaled size.
        src_dims.x > dest_dims.x || src_dims.y > dest_dims.y
    }

    fn render_infobar(&mut self) -> Result<(), String> {
        let text = infobar::Text::update(&self.ui_state.mode, &self.paths);
        // Load the filename texture
        let filename_surface = self
            .screen
            .font
            .render(&text.information)
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
            .render(&text.mode)
            .blended(text_color())
            .map_err(|e| e.to_string())?;
        let index_texture = self
            .screen
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
        self.render_bar(dims)?;
        // Copy the text textures
        let y = (self.screen.canvas.viewport().height() - index_dimensions.height) as i32;
        if let Err(e) = self.screen.canvas.copy(
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
        if let Err(e) = self.screen.canvas.copy(
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

    fn render_help(&mut self) -> Result<(), String> {
        let text = help_text();
        let total_height = LINE_HEIGHT * text.len() as i32 + LINE_PADDING * (text.len() as i32 - 1);
        let mut y = (self.screen.canvas.viewport().height() as f32 / 2.0
            - total_height as f32 / 2.0) as i32;
        let w = {
            let surface = self
                .screen
                .mono_font
                .render(text[0])
                .blended(text_color())
                .map_err(|e| e.to_string())?;
            let texture = self
                .screen
                .texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            texture.query().width
        };
        // Draw the Box
        let dims = (total_height as u32, w);
        self.render_help_box(dims)?;
        // Draw the text
        for line in text {
            // Load the text texture
            let surface = self
                .screen
                .mono_font
                .render(line)
                .blended(text_color())
                .map_err(|e| e.to_string())?;
            let texture = self
                .screen
                .texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            let dimensions = texture.query();
            let x = (self.screen.canvas.viewport().width() as f32 / 2.0
                - dimensions.width as f32 / 2.0) as i32;
            if let Err(e) = self.screen.canvas.copy(
                &texture,
                None,
                Rect::new(x, y, dimensions.width, dimensions.height),
            ) {
                eprintln!("Failed to copy text to screen {}", e);
            } else {
                y += LINE_HEIGHT + LINE_PADDING;
            }
        }
        Ok(())
    }

    fn render_bar(&mut self, dims: (u32, u32, u32)) -> Result<(), String> {
        let height = dims.0;
        let width = self.screen.canvas.viewport().width();
        let y = (self.screen.canvas.viewport().height() - height) as i32;
        let mut x = 0;
        let mut w = dims.1 + HALF_PAD as u32 * 3;
        self.screen.canvas.set_draw_color(light_blue());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = dims.2 + PADDING as u32 * 2;
        self.screen.canvas.set_draw_color(blue());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        x += w as i32;
        w = width;
        self.screen.canvas.set_draw_color(grey());
        if let Err(e) = self.screen.canvas.fill_rect(Rect::new(x, y, w, height)) {
            eprintln!("Failed to draw bar {}", e);
        }
        Ok(())
    }

    fn render_help_box(&mut self, dims: (u32, u32)) -> Result<(), String> {
        let height = dims.0;
        let y = (self.screen.canvas.viewport().height() as f32 / 2.0 - height as f32 / 2.0) as i32;
        let x = (self.screen.canvas.viewport().width() as f32 / 2.0 - dims.1 as f32 / 2.0) as i32;
        let w = dims.1;
        self.screen.canvas.set_draw_color(help_background_color());
        self.screen.canvas.set_blend_mode(BlendMode::Blend);
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
        if self.ui_state.render_help {
            self.render_help()?;
        }
        self.screen.canvas.present();
        Ok(())
    }
}

fn dark_grey() -> Color {
    Color::RGB(45, 45, 45)
}

fn text_color() -> Color {
    Color::RGBA(52, 56, 56, 255)
}

fn help_background_color() -> Color {
    Color::RGBA(0, 223, 252, 200)
}

fn light_blue() -> Color {
    Color::RGB(0, 223, 252)
}

fn blue() -> Color {
    Color::RGB(0, 180, 204)
}

fn grey() -> Color {
    Color::RGB(52, 56, 56)
}

fn help_text() -> Vec<&'static str> {
    vec![
        "+------------------+--------------------------------------------------------+",
        "| Key              | Action                                                 |",
        "|------------------|--------------------------------------------------------|",
        "| Esc OR q         | Quit                                                   |",
        "| Left Arrow OR k  | Previous Image                                         |",
        "| Right Arrow OR j | Next Image                                             |",
        "| PageUp OR w      | Forward 10% of images                                  |",
        "| PageDown OR b    | Backward 10% of images                                 |",
        "| Home OR g        | First Image                                            |",
        "| End OR G         | Last Image                                             |",
        "| m                | Move image to destination folder (default is ./keep)   |",
        "| c                | Copy image to destination folder (default is ./keep)   |",
        "| Delete OR d      | Delete image from it's location                        |",
        "| t                | Toggle information bar                                 |",
        "| f OR F11         | Toggle fullscreen mode                                 |",
        "| h                | Toggle help box                                        |",
        "| z OR Left Click  | Toggle actual size vs scaled image                     |",
        "| . (period)       | Repeat last action                                     |",
        "+------------------+--------------------------------------------------------+",
    ]
}
