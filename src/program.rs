use crate::cli;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::Sdl;
use sdl2::rect::Rect;
use std::path::PathBuf;
use std::time::Duration;

pub struct Program {
    sdl_context: Sdl,
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    images: Vec<PathBuf>,
    index: usize,
}

impl Program {
    pub fn init() -> Result<Program, String> {
        let images = cli::cli()?;
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
        let window = video
            .window("rust-sdl2 demo: Video", 
            video.display_bounds(0).unwrap().width(), 
            video.display_bounds(0).unwrap().height())
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        Ok(Program {
            sdl_context,
            canvas,
            texture_creator,
            images,
            index: 0,
        })
    }

    pub fn render(&mut self) -> Result<(), String> {
        let texture = self
            .texture_creator
            .load_texture(&self.images[self.index])?;
        let query = texture.query();
        let target = self.canvas.viewport();
        let dest = make_dst(query.width, query.height, target.width(), target.height());
        self.canvas.clear();
        self.canvas.copy(&texture, None, dest)?;
        self.canvas.present();
        Ok(())
    }

    fn increment(&mut self) -> Result<(), String> {
        if self.images.len() == 0 || self.images.len() == 1 {
            return Ok(());
        }
        if self.index <= self.images.len() - 2 {
            self.index += 1;
        }
        self.render()?;
        Ok(())
    }

    fn decrement(&mut self) -> Result<(), String> {
        if self.index > 0 {
            self.index -= 1;
        }
        self.render()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.render()?;

        'mainloop: loop {
            for event in self.sdl_context.event_pump()?.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Option::Some(Keycode::Escape),
                        ..
                    } => break 'mainloop,
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => self.decrement()?,
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => self.increment()?,

                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }
}

fn make_dst(src_x: u32, src_y: u32, dst_x: u32, dst_y: u32) -> Rect {
    match src_x > src_y {
        true => {
            match src_x > dst_x {
                true => {
                    let height = ((src_y as f32 / src_x as f32) * dst_x as f32) as u32;
                    let y = ((dst_y - height) as f32/2.0) as i32;
                    Rect::new(0, y, dst_x, height)
                },
                false => {
                    let y = ((dst_y - src_y) as f32/2.0) as i32;
                    let x = ((dst_x - src_x) as f32/2.0) as i32;
                    Rect::new(x, y, src_x, src_y)
                },
            }
        },
        false => {
            match src_y > dst_y {
                true => {
                    let width = ((src_y as f32 / src_x as f32) * dst_y as f32) as u32;
                    let x = ((dst_x - width) as f32/2.0) as i32;
                    Rect::new(x, 0, width, dst_y)
                },
                false => {
                    let y = ((dst_y - src_y) as f32/2.0) as i32;
                    let x = ((dst_x - src_x) as f32/2.0) as i32;
                    Rect::new(x, y, src_x, src_y)
                },
            }
        }
    }
}