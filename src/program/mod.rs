//! # Program
//!
//! Program contains the program struct, which contains all information needed to run the
//! event loop and render the images to screen

mod command_mode;
mod render;
pub use self::render::*;
use crate::cli;
use crate::paths::Paths;
use crate::screen::Screen;
use crate::sort::Sorter;
use crate::ui::{self, Action, Mode};
use core::cmp;
use fs_extra::file::copy;
use fs_extra::file::move_file;
use fs_extra::file::remove;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

const FONT_SIZE: u16 = 18;

/// Program contains all information needed to run the event loop and render the images to screen
pub struct Program<'a> {
    screen: Screen<'a>,
    paths: Paths,
    ui_state: ui::State<'a>,
    sorter: Sorter,
}

impl<'a> Program<'a> {
    /// init scaffolds the program, by making a call to the cli module to parse the command line
    /// arguments, sets up the sdl context, creates the window, the canvas and the texture
    /// creator.
    pub fn init(
        ttf_context: &'a Sdl2TtfContext,
        sdl_context: Sdl,
        canvas: Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        args: cli::Args,
    ) -> Result<Program<'a>, String> {
        let mut images = args.files;
        let dest_folder = args.dest_folder;
        let reverse = args.reverse;
        let sort_order = args.sort_order;
        let max_length = args.max_length;

        let max_viewable = if max_length > 0 && max_length <= images.len() {
            max_length
        } else {
            images.len()
        };

        let sorter = Sorter::new(sort_order, reverse);
        sorter.sort(&mut images);

        let current_dir = match std::env::current_dir() {
            Ok(c) => c,
            Err(_) => PathBuf::new(),
        };
        let font_bytes = include_bytes!("../../resources/Roboto-Medium.ttf");
        let font_bytes = match RWops::from_bytes(font_bytes) {
            Ok(b) => b,
            Err(e) => panic!("Failed to load font {}", e),
        };
        let font = match ttf_context.load_font_from_rwops(font_bytes, FONT_SIZE) {
            Ok(f) => f,
            Err(e) => panic!("Failed to load font {}", e),
        };
        let mono_font_bytes = include_bytes!("../../resources/RobotoMono-Medium.ttf");
        let mono_font_bytes = match RWops::from_bytes(mono_font_bytes) {
            Ok(b) => b,
            Err(e) => panic!("Failed to load font {}", e),
        };
        let mono_font = match ttf_context.load_font_from_rwops(mono_font_bytes, FONT_SIZE) {
            Ok(f) => f,
            Err(e) => panic!("Failed to load font {}", e),
        };
        Ok(Program {
            screen: Screen {
                sdl_context,
                canvas,
                texture_creator,
                font,
                mono_font,
                last_index: 0,
                last_texture: None,
                dirty: false,
            },
            paths: Paths {
                images,
                dest_folder,
                index: 0,
                current_dir,
                max_viewable,
                actual_max_viewable: max_length,
            },
            ui_state: ui::State {
                left_shift: false,
                right_shift: false,
                render_infobar: true,
                render_help: false,
                actual_size: false,
                fullscreen: args.fullscreen,
                mode: Mode::Normal,
                last_action: Action::Noop,
            },
            sorter,
        })
    }

    /// Toggle whether actual size or scaled image is rendered.
    pub fn toggle_fit(&mut self) {
        self.ui_state.actual_size = !self.ui_state.actual_size;
    }

    fn increment(&mut self, step: usize) -> Result<(), String> {
        if self.paths.images.is_empty() || self.paths.max_viewable == 1 {
            return Ok(());
        }
        if self.paths.index < self.paths.max_viewable - step {
            self.paths.index += step;
        }
        // Cap index at last image
        else {
            self.paths.index = self.paths.max_viewable - 1;
        }
        self.render_screen(false)
    }

    /// Removes an image from tracked images.
    /// Upholds that the index should always be <= index of last image.
    ///
    /// # Panics
    ///
    /// Panics if `index` tries to access past `self.images` bounds
    fn remove_image(&mut self, index: usize) {
        // Remove image
        // Panics if index is past bounds of vec
        self.paths.images.remove(index);
        // Adjust index if past bounds
        if index >= self.paths.max_viewable && self.paths.index != 0 {
            self.paths.index -= 1;
        }
    }

    fn decrement(&mut self, step: usize) -> Result<(), String> {
        if self.paths.index >= step {
            self.paths.index -= step;
        }
        // Step sizes bigger than remaining index are set to first image.
        else {
            self.paths.index = 0;
        }
        self.render_screen(false)
    }

    /// Skips forward by the default skip increment and renders the image
    pub fn skip_forward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(&self.paths.images);
        self.increment(skip_size)
    }

    /// Skips backward by the default skip increment and renders the image
    fn skip_backward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(&self.paths.images);
        self.decrement(skip_size)
    }

    /// Go to and render first image in list
    fn first(&mut self) -> Result<(), String> {
        self.paths.index = 0;
        self.render_screen(false)
    }

    /// Go to and render last image in list
    fn last(&mut self) -> Result<(), String> {
        if self.paths.images.is_empty() {
            self.paths.index = 0;
        } else {
            self.paths.index = self.paths.max_viewable - 1;
        }
        self.render_screen(false)
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
                self.paths.index, self.paths.max_viewable
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
        assert!(self.paths.index < self.paths.max_viewable);
        let current_imagepath = self.paths.images.get(self.paths.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.paths.index, self.paths.max_viewable
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
        self.remove_image(self.paths.index);
        self.screen.dirty = true;

        // Moving the image automatically advanced to next image
        // Adjust our view to reflect this
        self.render_screen(false)
    }

    /// Deletes image currently being viewed
    fn delete_image(&mut self) -> Result<(), String> {
        // Check if there is an image to delete
        if self.paths.images.is_empty() {
            return Err("no images to delete".to_string());
        }

        // Retrieve current image
        assert!(self.paths.index < self.paths.max_viewable);
        let current_imagepath = self.paths.images.get(self.paths.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.paths.index, self.paths.max_viewable
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
        self.remove_image(self.paths.index);
        self.screen.dirty = true;

        // Removing the image automatically advanced to next image
        // Adjust our view to reflect this
        self.render_screen(false)
    }

    /// Toggles fullscreen state of app
    pub fn toggle_fullscreen(&mut self) {
        self.ui_state.fullscreen = !self.ui_state.fullscreen;
    }

    /// Central run function that starts by default in Normal mode
    /// Switches modes allowing events to be interpreted in different ways
    pub fn run(&mut self) -> Result<(), String> {
        self.render_screen(false)?;
        'main_loop: loop {
            let mode = &self.ui_state.mode.clone();
            match mode {
                Mode::Normal => {
                    self.run_normal_mode()?;
                    self.render_screen(true)?;
                }
                Mode::Command(..) => {
                    self.run_command_mode()?;
                    // Force renders in order to remove "Command" and other info from bar
                    self.render_screen(true)?;
                }
                Mode::Error(..) => {
                    self.render_screen(false)?;
                    self.ui_state.mode = Mode::Normal;
                }
                Mode::Exit => break 'main_loop,
            }
        }
        Ok(())
    }

    /// run_normal_mode is the event loop that listens for input and delegates accordingly for
    /// normal mode
    fn run_normal_mode(&mut self) -> Result<(), String> {
        'mainloop: loop {
            for event in self.screen.sdl_context.event_pump()?.poll_iter() {
                match ui::process_normal_mode(&mut self.ui_state, &event) {
                    Action::Quit => {
                        self.ui_state.mode = Mode::Exit;
                        break 'mainloop;
                    }
                    Action::ToggleFullscreen => {
                        self.toggle_fullscreen();
                        self.screen.update_fullscreen(self.ui_state.fullscreen)?;
                        self.render_screen(false)?
                    }
                    Action::ReRender => self.render_screen(false)?,
                    Action::SwitchCommandMode => {
                        self.ui_state.mode = Mode::Command(String::new());
                        break 'mainloop;
                    }
                    Action::ToggleFit => {
                        self.toggle_fit();
                        self.render_screen(false)?
                    }
                    Action::Next => self.increment(1)?,
                    Action::Prev => self.decrement(1)?,
                    Action::First => self.first()?,
                    Action::Last => self.last()?,
                    Action::SkipForward => self.skip_forward()?,
                    Action::SkipBack => self.skip_backward()?,
                    Action::Copy => match self.copy_image() {
                        Ok(_) => (),
                        Err(e) => eprintln!("Failed to copy file: {}", e),
                    },
                    Action::Move => match self.move_image() {
                        Ok(_) => (),
                        Err(e) => eprintln!("Failed to move file: {}", e),
                    },
                    Action::Delete => match self.delete_image() {
                        Ok(_) => (),
                        Err(e) => eprintln!("{}", e),
                    },
                    Action::Noop => {}
                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(1000 / 60));
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

/// Creates a rectangle which is centered on the src dimensions.
/// For each src dimension, if the src is larger than the destination dimension, the
/// rectangle is capped at the destination dimension.
fn compute_center_rectangle_view(src_width: u32, src_height: u32, target_rect: &Rect) -> Rect {
    let tex_center = calculate_texture_center(src_width, src_height);

    // create centered rectangle for texture
    // Don't extend past max dimensions of src texture
    let target_width = target_rect.width();
    let target_height = target_rect.height();
    let clip_width = if src_width > target_width {
        target_width
    } else {
        src_width
    };
    let clip_height = if src_height > target_height {
        target_height
    } else {
        src_height
    };

    // Centered slice which fits within destination boundaries
    Rect::from_center(tex_center, clip_width, clip_height)
}

/// Primarily used for finding the center of a Texture.
/// Computes the center of a rectangle, given the x and y points
/// of the top-right corner of the rectangle.
fn calculate_texture_center(src_x: u32, src_y: u32) -> Point {
    Rect::new(0, 0, src_x, src_y).center()
}
