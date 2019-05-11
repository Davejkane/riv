//! # Program
//!
//! Program contains the program struct, which contains all information needed to run the
//! event loop and render the images to screen

use crate::cli;
use crate::ui::{self, Action};
use core::cmp;
use fs_extra::file::copy;
use fs_extra::file::move_file;
use fs_extra::file::remove;
use sdl2::image::LoadTexture;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, TextureQuery, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::Sdl;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

/// Compute increment of skips
/// Does not account for overflow or underflow of vector
fn compute_skip_size(images: &[PathBuf]) -> usize {
    let chunks = 10usize;
    let skip_size: usize = (images.len() as usize / chunks) as usize + 1usize;

    // Skip increment must be at least 1
    cmp::max(1usize, skip_size)
}

/// Program contains all information needed to run the event loop and render the images to screen
pub struct Program {
    sdl_context: Sdl,
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    images: Vec<PathBuf>,
    dest_folder: PathBuf,
    index: usize,
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
            ..Default::default()
        };
        Ok(Program {
            sdl_context,
            canvas,
            texture_creator,
            images,
            dest_folder,
            index: 0,
            ui_state,
        })
    }

    ///Toggle whether the viewport uses the actual size of the image
    pub fn toggle_fit(&mut self) {
        self.ui_state.actual_size = !self.ui_state.actual_size;
    }

    /// render loads the image at the path in the images path vector located at the index and
    /// renders to screen
    pub fn render(&mut self) -> Result<(), String> {
        if self.images.is_empty() {
            return self.render_blank();
        }
        let texture = match self.texture_creator.load_texture(&self.images[self.index]) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("failed to render image {}", e);
                return Ok(());
            }
        };
        let query: TextureQuery = texture.query();
        // Target: Place to render
        let target = self.canvas.viewport();
        let src_slice = compute_center_rectangle_view(query.width, query.height, target);

        dbg!(src_slice);
        //

        // Dest: Portion of target to render on
        let dest = if self.ui_state.actual_size {
            make_dst(
                src_slice.width(),
                src_slice.height(),
                target.width(),
                target.height(),
            )
        } else {
            make_dst(query.width, query.height, target.width(), target.height())
        };
        dbg!(query);
        dbg!(target);
        dbg!(dest);
        self.canvas.clear();
        dbg!(self.ui_state.actual_size);
        if self.ui_state.actual_size {
            // Copy only portion of texture which fits inside view
            if let Err(e) = self.canvas.copy(&texture, src_slice, dest) {
                eprintln!("Failed to copy image to screen {}", e);
                return Ok(());
            }
        } else {
            if let Err(e) = self.canvas.copy(&texture, None, dest) {
                eprintln!("Failed to copy image to screen {}", e);
                return Ok(());
            }
        }
        self.canvas.present();
        Ok(())
    }

    fn render_blank(&mut self) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.present();
        Ok(())
    }
}

fn compute_center_rectangle_view(src_width: u32, src_height: u32, target_rect: Rect) -> Rect {
    let tex_center = calculate_texture_center(src_width, src_height);

    // create centered rectangle for texture
    // Don't extend past max dimentions of src texture
    let target_width = target_rect.width();
    let target_height = target_rect.height();
    let copy_width_boundry = if src_width > target_width {
        target_width
    } else {
        src_width
    };
    let copy_height_boundry = if src_height > target_height {
        target_height
    } else {
        src_height
    };

    let center_texture_rect =
        Rect::from_center(tex_center, copy_width_boundry, copy_height_boundry);
    center_texture_rect
}

fn calculate_texture_center(src_x: u32, src_y: u32) -> Point {
    Rect::new(0, 0, src_x, src_y).center()
}

impl Program {
    fn increment(&mut self, step: usize) -> Result<(), String> {
        if self.images.is_empty() || self.images.len() == 1 {
            return Ok(());
        }
        if self.index < self.images.len() - step {
            self.index += step;
        }
        // Cap index at last image
        else {
            self.index = self.images.len() - 1;
        }
        self.render()
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
        self.images.remove(index);
        // Adjust index if past bounds
        if index >= self.images.len() && self.index != 0 {
            self.index -= 1;
        }
    }

    fn decrement(&mut self, step: usize) -> Result<(), String> {
        if self.index >= step {
            self.index -= step;
        }
        // Step sizes bigger than remaining index are set to first image.
        else {
            self.index = 0;
        }
        self.render()
    }

    /// Returns new index to advance to
    pub fn skip_forward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(&self.images);
        self.increment(skip_size)
    }

    /// Returns new index to skip back to
    fn skip_backward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(&self.images);
        self.decrement(skip_size)
    }

    fn first(&mut self) -> Result<(), String> {
        self.index = 0;
        self.render()
    }

    fn last(&mut self) -> Result<(), String> {
        if self.images.is_empty() {
            self.index = 0;
        } else {
            self.index = self.images.len() - 1;
        }
        self.render()
    }

    fn construct_dest_filepath(&self, src_path: &PathBuf) -> Result<PathBuf, String> {
        match std::fs::create_dir_all(&self.dest_folder) {
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
        let newname = PathBuf::from(&self.dest_folder).join(cur_filename);
        Ok(newname)
    }

    /// Copies currently rendered image to dest directory
    /// TODO: Handle when file already exists in dest directory
    fn copy_image(&mut self) -> Result<(), String> {
        // Check if there are any images
        if self.images.is_empty() {
            return Err("No image to copy".to_string());
        }
        let opt = &fs_extra::file::CopyOptions::new();
        let filepath = self.images.get(self.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.index,
                self.images.len()
            ))
        });
        let newname = self.construct_dest_filepath(filepath)?;
        copy(filepath, newname, opt).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Moves image currently being viewed to destination folder
    fn move_image(&mut self) -> Result<(), String> {
        // Check if there is an image to move
        if self.images.is_empty() {
            return Err("no images to move".to_string());
        }
        // Retrieve current image
        assert!(self.index < self.images.len());
        let current_imagepath = self.images.get(self.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.index,
                self.images.len()
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
        self.remove_image(self.index);

        // Moving the image automatically advanced to next image
        // Adjust our view to reflect this
        self.render()
    }

    /// Deletes image currently being viewed
    fn delete_image(&mut self) -> Result<(), String> {
        // Check if there is an image to delete
        if self.images.is_empty() {
            return Err("no images to delete".to_string());
        }

        // Retrieve current image
        assert!(self.index < self.images.len());
        let current_imagepath = self.images.get(self.index).unwrap_or_else(|| {
            panic!(format!(
                "image index {} > max image index {}",
                self.index,
                self.images.len()
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
        self.remove_image(self.index);

        // Removing the image automatically advanced to next image
        // Adjust our view to reflect this
        self.render()
    }

    /// run is the event loop that listens for input and delegates accordingly.
    pub fn run(&mut self) -> Result<(), String> {
        self.render()?;

        'mainloop: loop {
            for event in self.sdl_context.event_pump()?.poll_iter() {
                match ui::event_action(&mut self.ui_state, &event) {
                    Action::Quit => break 'mainloop,
                    Action::ReRender => self.render()?,
                    Action::ToggleFit => {
                        self.toggle_fit();
                        self.render()?
                    }
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
        full_rect(src_x, src_y, dst_x, dst_y)
    }
    // case 2: source aspect ratio is larger
    else if src_x as f32 / src_y as f32 > dst_x as f32 / dst_y as f32 {
        fit_x_rect(src_x, src_y, dst_x, dst_y)
    }
    // case 3: source aspect ratio is smaller
    else {
        fit_y_rect(src_x, src_y, dst_x, dst_y)
    }
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

#[cfg(test)]
mod tests {
    use super::Point;
    use super::Rect;
    use crate::program::compute_center_rectangle_view;

    #[test]
    fn test_center_content_same_area_both() {
        let src_texture_rect = Rect::new(0, 0, 1000, 1000);
        let target_area = Rect::new(0, 0, 1000, 1000);

        let src_slice = compute_center_rectangle_view(
            src_texture_rect.width(),
            src_texture_rect.height(),
            target_area,
        );
        let src_slice_center = src_slice.center();
        assert_eq!(src_slice_center, Point::new(500, 500));
        assert_eq!(src_slice.width(), 1000);
        assert_eq!(src_slice.height(), 1000);
    }

    #[test]
    fn test_center_content_texture_xdim_bigger_than_viewport_x() {
        let src_texture_rect = Rect::new(0, 0, 2000, 1000);
        let target_area = Rect::new(0, 0, 1000, 1000);

        let src_slice = compute_center_rectangle_view(
            src_texture_rect.width(),
            src_texture_rect.height(),
            target_area,
        );
        let src_slice_center = src_slice.center();
        assert_eq!(src_slice_center, Point::new(500, 1000));
        assert_eq!(src_slice.width(), 1000);
        assert_eq!(src_slice.height(), 1000);
    }
}
