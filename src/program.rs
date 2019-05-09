//! # Program
//!
//! Program contains the program struct, which contains all information needed to run the
//! event loop and render the images to screen

use crate::cli;
use crate::paths::Paths;
use crate::screen::Screen;
use crate::ui::{self, Action};
use core::cmp;
use fs_extra::file::copy;
use fs_extra::file::move_file;
use fs_extra::file::remove;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

/// Program contains all information needed to run the event loop and render the images to screen
pub struct Program {
    screen: Screen,
    paths: Paths,
    ui_state: ui::State,
}

impl Program {
    /// init scaffolds the program, by making a call to the cli module to parse the command line
    /// arguments, sets up the sdl context, creates the window, the canvas and the texture
    /// creator.
    pub fn init() -> Result<Program, String> {
        let args = cli::cli()?;
        let images = args.files;
        let dest_folder = args.dest_folder;
        let glob = args.search;
        let current_dir = match std::env::current_dir() {
            Ok(c) => c,
            Err(_) => PathBuf::new(),
        };
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
        let ui_state = ui::State {
            left_shift: false,
            right_shift: false,
        };
        Ok(Program {
            screen: Screen {
                sdl_context,
                canvas,
                texture_creator,
            },
            paths: Paths {
                images,
                dest_folder,
                index: 0,
                glob,
                current_dir,
            },
            ui_state,
        })
    }

    /// render loads the image at the path in the images path vector located at the index and
    /// renders to screen
    pub fn render(&mut self) -> Result<(), String> {
        if self.paths.images.is_empty() {
            return self.render_blank();
        }
        let texture = match self
            .screen
            .texture_creator
            .load_texture(&self.paths.images[self.paths.index])
        {
            Ok(t) => t,
            Err(e) => {
                eprintln!("failed to render image {}", e);
                return Ok(());
            }
        };
        let query = texture.query();
        let target = self.screen.canvas.viewport();
        let dest = make_dst(query.width, query.height, target.width(), target.height());
        self.screen.canvas.clear();
        if let Err(e) = self.screen.canvas.copy(&texture, None, dest) {
            eprintln!("Failed to copy image to screen {}", e);
            return Ok(());
        }
        self.screen.canvas.present();
        Ok(())
    }

    fn render_blank(&mut self) -> Result<(), String> {
        self.screen.canvas.clear();
        self.screen.canvas.present();
        Ok(())
    }

    fn increment(&mut self, step: usize) -> Result<(), String> {
        if self.paths.images.is_empty() || self.paths.images.len() == 1 {
            return Ok(());
        }
        if self.paths.index < self.paths.images.len() - step {
            self.paths.index += step;
        }
        // Cap index at last image
        else {
            self.paths.index = self.paths.images.len() - 1;
        }
        self.render()
    }

    fn decrement(&mut self, step: usize) -> Result<(), String> {
        if self.paths.index >= step {
            self.paths.index -= step;
        }
        // Step sizes bigger than remaining index are set to first image.
        else {
            self.paths.index = 0;
        }
        self.render()
    }

    /// Returns new index to advance to
    pub fn skip_forward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(&self.paths.images);
        self.increment(skip_size)
    }

    /// Returns new index to skip back to
    fn skip_backward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(&self.paths.images);
        self.decrement(skip_size)
    }

    fn first(&mut self) -> Result<(), String> {
        self.paths.index = 0;
        self.render()
    }

    fn last(&mut self) -> Result<(), String> {
        if self.paths.images.is_empty() {
            self.paths.index = 0;
        } else {
            self.paths.index = self.paths.images.len() - 1;
        }
        self.render()
    }

    fn construct_dest_filepath(&self, src_path: &PathBuf) -> Result<PathBuf, String> {
        match std::fs::create_dir_all(&self.paths.dest_folder) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                ErrorKind::AlreadyExists => (),
                _ => return Err(e.to_string()),
            },
        };

        let cur_filename = match src_path.file_name() {
            Some(f) => f,
            None => return Err("failed to read filename for current image".to_string()),
        };
        let newname = PathBuf::from(&self.paths.dest_folder).join(cur_filename);
        Ok(newname)
    }

    /// Copies currently rendered image to dest directory
    /// TODO: Handle when file already exists in dest directory
    fn copy_image(&mut self) -> Result<(), String> {
        // Check if there are any images
        if self.paths.images.is_empty() {
            return Err("No image to copy".to_string());
        }
        let opt = &fs_extra::file::CopyOptions::new();
        let filepath = self.paths.images.get(self.paths.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.paths.index,
                self.paths.images.len()
            ))
        });
        let newname = self.construct_dest_filepath(filepath)?;
        copy(filepath, newname, opt).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Moves image currently being viewed to destination folder
    fn move_image(&mut self) -> Result<(), String> {
        // Check if there is an image to move
        if self.paths.images.is_empty() {
            return Err("no images to move".to_string());
        }
        // Retrieve current image
        assert!(self.paths.index < self.paths.images.len());
        let current_imagepath = self.paths.images.get(self.paths.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.paths.index,
                self.paths.images.len()
            ))
        });

        let newname = self.construct_dest_filepath(&current_imagepath)?;
        let opt = &fs_extra::file::CopyOptions::new();

        // Attempt to move image
        if let Err(e) = move_file(current_imagepath, newname, opt) {
            return Err(format!(
                "Failed to remove image `{:?}`: {}",
                current_imagepath,
                e.to_string()
            ));
        }

        // Only if successful, remove image from tracked images
        self.paths.images.remove(self.paths.index);

        // Adjust our view
        self.decrement(1)
    }

    /// Deletes image currently being viewed
    fn delete_image(&mut self) -> Result<(), String> {
        // Check if there is an image to delete
        if self.paths.images.is_empty() {
            return Err("no images to delete".to_string());
        }

        // Retrieve current image
        assert!(self.paths.index < self.paths.images.len());
        let current_imagepath = self.paths.images.get(self.paths.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.paths.index,
                self.paths.images.len()
            ))
        });

        // Attempt to remove image
        if let Err(e) = remove(&current_imagepath) {
            return Err(format!(
                "Failed to remove image `{:?}`: {}",
                current_imagepath,
                e.to_string()
            ));
        }
        // If we've reached past here, there was no error deleting the image

        // Only if successful, remove image from tracked images
        self.paths.images.remove(self.paths.index);

        // Adjust our view
        self.decrement(1)
    }

    /// run is the event loop that listens for input and delegates accordingly.
    pub fn run(&mut self) -> Result<(), String> {
        self.render()?;

        'mainloop: loop {
            for event in self.screen.sdl_context.event_pump()?.poll_iter() {
                match ui::event_action(&mut self.ui_state, &event) {
                    Action::Quit => break 'mainloop,
                    Action::ReRender => self.render()?,
                    Action::Next => self.increment(1)?,
                    Action::Prev => self.decrement(1)?,
                    Action::Copy => match self.copy_image() {
                        Ok(_) => (),
                        Err(e) => eprintln!("Failed to copy file: {}", e),
                    },
                    Action::Move => match self.move_image() {
                        Ok(_) => (),
                        Err(e) => eprintln!("Failed to move file: {}", e),
                    },
                    Action::SkipForward => self.skip_forward()?,
                    Action::SkipBack => self.skip_backward()?,
                    Action::Delete => match self.delete_image() {
                        Ok(_) => (),
                        Err(e) => eprintln!("{}", e),
                    },
                    Action::First => self.first()?,
                    Action::Last => self.last()?,
                    Action::Noop => {}
                }
            }
            std::thread::sleep(Duration::from_millis(0));
        }

        Ok(())
    }
}

/// make dst determines the parameters of a rectangle required to place an image correctly in
/// the window
fn make_dst(src_x: u32, src_y: u32, dst_x: u32, dst_y: u32) -> Rect {
    // case 1: both source dimensions smaller
    if src_x < dst_x && src_y < dst_y {
        return full_rect(src_x, src_y, dst_x, dst_y);
    }
    // case 2: source aspect ratio is larger
    if src_x as f32 / src_y as f32 > dst_x as f32 / dst_y as f32 {
        return fit_x_rect(src_x, src_y, dst_x, dst_y);
    }
    // case 3: source aspect ratio is smaller
    fit_y_rect(src_x, src_y, dst_x, dst_y)
}

fn full_rect(src_x: u32, src_y: u32, dst_x: u32, dst_y: u32) -> Rect {
    let y = ((dst_y - src_y) as f32 / 2.0) as i32;
    let x = ((dst_x - src_x) as f32 / 2.0) as i32;
    Rect::new(x, y, src_x, src_y)
}

fn fit_x_rect(src_x: u32, src_y: u32, dst_x: u32, dst_y: u32) -> Rect {
    let height = ((src_y as f32 / src_x as f32) * dst_x as f32) as u32;
    let y = ((dst_y - height) as f32 / 2.0) as i32;
    Rect::new(0, y, dst_x, height)
}

fn fit_y_rect(src_x: u32, src_y: u32, dst_x: u32, dst_y: u32) -> Rect {
    let width = ((src_x as f32 / src_y as f32) * dst_y as f32) as u32;
    let x = ((dst_x - width) as f32 / 2.0) as i32;
    Rect::new(x, 0, width, dst_y)
}

/// Compute increment of skips
/// Does not account for overflow or underflow of vector
fn compute_skip_size(images: &[PathBuf]) -> usize {
    let chunks = 10usize;
    let skip_size: usize = (images.len() as usize / chunks) as usize + 1usize;

    // Skip increment must be at least 1
    cmp::max(1usize, skip_size)
}
