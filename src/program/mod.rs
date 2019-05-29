//! # Program
//!
//! Program contains the program struct, which contains all information needed to run the
//! event loop and render the images to screen

mod command_mode;
mod render;
pub use self::render::*;
use crate::cli;
use crate::paths::{Paths, PathsBuilder};
use crate::screen::Screen;
use crate::sort::Sorter;
use crate::ui::{self, Action, Mode, PanAction, ProcessAction, ZoomAction};
use core::cmp;
use fs_extra::file::copy;
use fs_extra::file::move_file;
use fs_extra::file::remove;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::rwops::RWops;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const FONT_SIZE: u16 = 18;
const PAN_PIXELS: f32 = 50.0;

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

        let max_viewable = max_length;

        let sorter = Sorter::new(sort_order, reverse);
        sorter.sort(&mut images);

        let base_dir = match std::env::current_dir() {
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

        let paths = PathsBuilder::new(images, dest_folder, base_dir)
            .with_maximum_viewable(max_viewable)
            .build();
        Ok(Program {
            screen: Screen {
                sdl_context,
                canvas,
                texture_creator,
                font,
                mono_font,
                last_index: None,
                last_texture: None,
                dirty: false,
            },
            paths,
            ui_state: ui::State {
                fullscreen: args.fullscreen,
                ..Default::default()
            },
            sorter,
        })
    }

    /// Toggle whether actual size or scaled image is rendered.
    pub fn toggle_fit(&mut self) -> Result<(), String> {
        let error = 0.001;
        if (self.ui_state.scale - 1.0).abs() > error {
            self.ui_state.scale = 1.0;
        } else {
            self.ui_state.scale = self.calculate_scale_for_fit();
        }
        self.render_screen(false)
    }

    /// Centres the image after any panning has taken place
    pub fn center_image(&mut self) -> Result<(), String> {
        self.ui_state.pan_x = 0.0;
        self.ui_state.pan_y = 0.0;
        self.render_screen(false)
    }

    // Calculates the scale required to fit large images to screen
    fn calculate_scale_for_fit(&self) -> f32 {
        if let Some(tex) = self.screen.last_texture.as_ref() {
            let query = tex.query();
            let (src_x, src_y) = (query.width, query.height);
            let target = self.screen.canvas.viewport();
            let (dst_x, dst_y) = (target.width(), target.height());
            // case 1: both source dimensions smaller
            if src_x < dst_x && src_y < dst_y {
                return 1.0;
            }
            // case 2: source aspect ratio is larger
            if src_x as f32 / src_y as f32 > dst_x as f32 / dst_y as f32 {
                return dst_x as f32 / src_x as f32;
            }
            // case 3: source aspect ratio is smaller
            dst_y as f32 / src_y as f32
        } else {
            1.0
        }
    }

    fn increment(&mut self, step: usize) -> Result<(), String> {
        self.paths.increment(step);
        self.render_screen(false)
    }

    /// Moves tracking current image down by `step`
    fn decrement(&mut self, step: usize) -> Result<(), String> {
        self.paths.decrement(step);
        self.render_screen(false)
    }

    /// Skips forward by the default skip increment and renders the image
    pub fn skip_forward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(self.paths.images());
        self.increment(skip_size)
    }

    /// Skips backward by the default skip increment and renders the image
    fn skip_backward(&mut self) -> Result<(), String> {
        let skip_size = compute_skip_size(self.paths.images());
        self.decrement(skip_size)
    }

    /// Go to and render first image in list
    fn first(&mut self) -> Result<(), String> {
        // If there is at least one image
        match self.paths.index() {
            Some(_) => {
                // Set the current image to the first index
                self.paths.set_index(0);
                self.render_screen(false)
            }
            None => {
                // Nothing to do
                Ok(())
            }
        }
    }

    /// Go to and render last image in list
    fn last(&mut self) -> Result<(), String> {
        // If there is at least one image
        if let Some(last) = self.paths.max_viewable_index() {
            // Set the current image to the last viewable index
            self.paths.set_index(last);
            self.render_screen(false)
        } else {
            // No images means no last index
            Ok(())
        }
    }

    /// Zooms in
    fn zoom_in(&mut self, times: usize) -> Result<(), String> {
        let zoom_factor: f32 = 1.1;
        let zoom_times = (times) as i32;
        // Malicious huge numbers
        let zoom_times = if zoom_times.is_positive() {
            zoom_times
        } else {
            1
        };
        self.ui_state.scale *= zoom_factor.powi(zoom_times);
        self.render_screen(false)
    }

    /// Zooms out
    fn zoom_out(&mut self, times: usize) -> Result<(), String> {
        let zoom_factor: f32 = 0.9;
        // When taking to a power, increase the times by 1
        let zoom_times = (times) as i32;
        // Malicious huge numbers or 0 check
        let zoom_times = if zoom_times.is_positive() {
            zoom_times
        } else {
            1
        };
        self.ui_state.scale *= zoom_factor.powi(zoom_times);
        self.render_screen(false)
    }

    /// Pans left
    fn pan_left(&mut self) -> Result<(), String> {
        let step = self.calc_x_step();
        self.ui_state.pan_x += step;
        if self.ui_state.pan_x > 1.0 {
            self.ui_state.pan_x = 1.0;
        }
        self.render_screen(false)
    }

    /// Pans right
    fn pan_right(&mut self) -> Result<(), String> {
        let step = self.calc_x_step();
        self.ui_state.pan_x -= step;
        if self.ui_state.pan_x < -1.0 {
            self.ui_state.pan_x = -1.0;
        }
        self.render_screen(false)
    }

    /// Pans up
    fn pan_up(&mut self) -> Result<(), String> {
        let step = self.calc_y_step();
        self.ui_state.pan_y += step;
        if self.ui_state.pan_y > 1.0 {
            self.ui_state.pan_y = 1.0;
        }
        self.render_screen(false)
    }

    /// Pans down
    fn pan_down(&mut self) -> Result<(), String> {
        let step = self.calc_y_step();
        self.ui_state.pan_y -= step;
        if self.ui_state.pan_y < -1.0 {
            self.ui_state.pan_y = -1.0;
        }
        self.render_screen(false)
    }

    fn calc_x_step(&self) -> f32 {
        if let Some(tex) = self.screen.last_texture.as_ref() {
            let src_w = tex.query().width;
            let dst_w = self.screen.canvas.viewport().width();
            let x_diff = (dst_w as f32 - (src_w as f32 * self.ui_state.scale)) / 2.0;
            (PAN_PIXELS / x_diff).abs()
        } else {
            0.0
        }
    }

    fn calc_y_step(&self) -> f32 {
        if let Some(tex) = self.screen.last_texture.as_ref() {
            let src_h = tex.query().height;
            let dst_h = self.screen.canvas.viewport().height();
            let y_diff = (dst_h as f32 - (src_h as f32 * self.ui_state.scale)) / 2.0;
            (PAN_PIXELS / y_diff).abs()
        } else {
            0.0
        }
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

    /// Copies the current image and (n-1) next images
    /// Does nothing if supplied 0 for an amount
    fn copy_images(&self, amount: usize) -> Result<String, String> {
        if amount == 0 {
            return Ok("0 images asked to copy".to_string());
        }

        let current_index = match self.paths.index() {
            Some(i) => i,
            None => return Err("no images to copy".to_string()),
        };

        let copy_range = current_index..=(current_index.saturating_add(amount - 1));
        let paths = self.paths.get_range(&copy_range);
        let paths = match paths {
            Some(paths) => paths,
            None => {
                return Err(format!(
                    "Image range {}..={} is out of range",
                    copy_range.start(),
                    copy_range.end()
                ))
            }
        };

        for imagepath in paths {
            let newname = self.construct_dest_filepath(imagepath)?;
            self.copy_image(imagepath, &newname)?;
        }

        Ok(format!(
            "copied {} image(s) to {} succesfully",
            paths.len(),
            self.paths.dest_folder.to_str().unwrap(),
        ))
    }

    /// Copies currently rendered image to dest directory
    /// TODO: Handle when file already exists in dest directory
    fn copy_current_image(&self) -> Result<String, String> {
        // Check if there are any images
        let filepath = match self.paths.current_image_path() {
            Some(path) => path,
            None => return Err("no images to copy".to_string()),
        };

        let newname = self.construct_dest_filepath(filepath)?;
        self.copy_image(filepath, &newname)?;

        Ok(format!(
            "copied image to {} succesfully",
            newname.to_str().unwrap()
        ))
    }

    /// Helper method to copy images
    fn copy_image(&self, src_file: &PathBuf, dest_file: &PathBuf) -> Result<(), String> {
        let opt = &fs_extra::file::CopyOptions::new();
        copy(src_file, dest_file, opt).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Moves image currently being viewed to destination folder
    fn move_image(&mut self) -> Result<String, String> {
        // Check if there is an image to move
        let current_imagepath = match self.paths.current_image_path() {
            Some(path) => path,
            None => return Err("no images to move".to_string()),
        };

        let success_msg = format!(
            "moved {} succesfully to {}",
            &current_imagepath.to_str().unwrap(),
            self.paths.dest_folder.to_str().unwrap()
        );

        let newname = self.construct_dest_filepath(&current_imagepath)?;
        let opt = &fs_extra::file::CopyOptions::new();

        // Attempt to move image
        if let Err(e) = move_file(&current_imagepath, newname, opt) {
            return Err(format!(
                "Failed to remove image `{:?}`: {}",
                &current_imagepath,
                e.to_string()
            ));
        }
        // Only if successful, remove image from tracked images
        self.paths.remove_current_image();
        self.screen.dirty = true;

        // Moving the image automatically advanced to next image
        // Adjust our view to reflect this
        self.render_screen(false)?;
        Ok(success_msg)
    }

    /// Deletes image currently being viewed
    fn delete_image(&mut self) -> Result<String, String> {
        // Check if there is an image to delete
        let current_imagepath = match self.paths.current_image_path() {
            Some(path) => path,
            None => return Err("no images to delete".to_string()),
        };

        let success_msg = format!(
            "deleted {} successfully",
            &current_imagepath.to_str().unwrap()
        );

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
        self.paths.remove_current_image();
        self.screen.dirty = true;

        // Removing the image automatically advanced to next image
        // Adjust our view to reflect this
        self.render_screen(false)?;
        Ok(success_msg)
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
                    // Don't reset image zoom and offset when changing modes
                    self.render_screen(false)?;
                }
                Mode::MultiNormal => {
                    self.run_multi_normal_mode()?;
                    // Keep image zoom and offset when going back to Normal mode
                    self.render_screen(false)?;
                }
                Mode::Command(..) => {
                    self.run_command_mode()?;
                    // Force renders in order to remove "Command" and other info from bar
                    self.render_screen(false)?;
                }
                Mode::Error(..) => {
                    self.render_screen(false)?;
                    self.ui_state.mode = Mode::Normal;
                }
                Mode::Success(..) => {
                    self.render_screen(false)?;
                    self.ui_state.mode = Mode::Normal;
                }
                Mode::Exit => break 'main_loop,
            }
        }
        Ok(())
    }

    /// Mode to input how many times to repeat a normal mode action
    /// Previous input from normal mode is in `self.ui_state.current_input`
    fn run_multi_normal_mode(&mut self) -> Result<(), String> {
        use ui::MultiNormalAction;

        let mut complete_action = false;
        while !complete_action {
            for event in self.screen.sdl_context.event_pump()?.poll_iter() {
                let multi_action = ui::process_multi_normal_mode(&mut self.ui_state, event);

                // Assume input is finished unless set by other actions
                complete_action = true;

                match multi_action {
                    MultiNormalAction::MoreInput => {
                        complete_action = false;
                    }
                    MultiNormalAction::Cancel => {
                        self.ui_state.mode = Mode::Normal;
                    }
                    MultiNormalAction::Repeat(process) => {

                        self.ui_state.process_action(process.action.clone());
                        match process {
                            ProcessAction {
                                action: a,
                                times: n,
                            } => match (a, n) {
                                (Action::Copy, n) => match self.copy_images(n) {
                                    Ok(s) => {
                                        self.ui_state.mode = Mode::Success(s);
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        self.ui_state.mode = Mode::Error(e);
                                        return Ok(());
                                    }
                                },
                                (Action::Zoom(ZoomAction::In), n) => self.zoom_in(n)?,
                                (Action::Zoom(ZoomAction::Out), n) => self.zoom_out(n)?,
                                (_, _) => println!("Not multi followed"),
                            },
                        }
                        self.ui_state.mode = Mode::Normal;
                        // clear repeat register
                        self.ui_state.repeat = 1;
                    }
                    MultiNormalAction::SwitchBackNormalMode => {
                        self.ui_state.mode = Mode::Normal;
                    }
                    MultiNormalAction::Quit => {
                        self.ui_state.mode = Mode::Exit;
                    }
                    _ => {},
                }
                std::thread::sleep(Duration::from_millis(1000 / 60));
            }
        }
        Ok(())
    }

    /// run_normal_mode is the event loop that listens for input and delegates accordingly for
    /// normal mode
    fn run_normal_mode(&mut self) -> Result<(), String> {
        'mainloop: loop {
            for event in self.screen.sdl_context.event_pump()?.poll_iter() {
                let action = ui::process_normal_mode(&mut self.ui_state, event);
                self.ui_state.process_action(action.clone());
                match action {
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
                    Action::SwitchMultiNormalMode => {
                        self.ui_state.mode = Mode::MultiNormal;
                        break 'mainloop;
                    }
                    Action::ToggleFit => self.toggle_fit()?,
                    Action::CenterImage => self.center_image()?,
                    Action::Next => self.increment(1)?,
                    Action::Prev => self.decrement(1)?,
                    Action::First => self.first()?,
                    Action::Last => self.last()?,
                    Action::SkipForward => self.skip_forward()?,
                    Action::SkipBack => self.skip_backward()?,
                    Action::Zoom(ZoomAction::In) => self.zoom_in(1)?,
                    Action::Zoom(ZoomAction::Out) => self.zoom_out(1)?,
                    Action::Pan(PanAction::Left) => self.pan_left()?,
                    Action::Pan(PanAction::Right) => self.pan_right()?,
                    Action::Pan(PanAction::Up) => self.pan_up()?,
                    Action::Pan(PanAction::Down) => self.pan_down()?,
                    Action::Copy => match self.copy_current_image() {
                        Ok(s) => {
                            self.ui_state.mode = Mode::Success(s);
                            self.ui_state.rerender_time = Some(Instant::now());
                            return Ok(());
                        }
                        Err(e) => {
                            self.ui_state.mode = Mode::Error(format!("Failed to copy file: {}", e));
                            return Ok(());
                        }
                    },
                    Action::Move => match self.move_image() {
                        Ok(s) => {
                            self.ui_state.mode = Mode::Success(s);
                            self.ui_state.rerender_time = Some(Instant::now());
                            return Ok(());
                        }
                        Err(e) => {
                            self.ui_state.mode = Mode::Error(format!("Failed to move file: {}", e));
                            return Ok(());
                        }
                    },
                    Action::Delete => match self.delete_image() {
                        Ok(s) => {
                            self.ui_state.mode = Mode::Success(s);
                            self.ui_state.rerender_time = Some(Instant::now());
                            return Ok(());
                        }
                        Err(e) => {
                            self.ui_state.mode =
                                Mode::Error(format!("Failed to delete file: {}", e));
                            return Ok(());
                        }
                    },
                    Action::Noop => {}
                    _ => {}
                }
            }
            if let Some(ts) = self.ui_state.rerender_time {
                if Instant::now().duration_since(ts) > Duration::from_millis(1500) {
                    self.ui_state.rerender_time = None;
                    self.render_screen(false)?;
                }
            }
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }

        Ok(())
    }
}

/// make dst determines the parameters of a rectangle required to place an image correctly in
/// the window
fn make_dst(tq: &TextureQuery, vp: &Rect, scale: f32, pan_x: f32, pan_y: f32) -> Rect {
    let x_diff = (vp.width() as f32 - (tq.width as f32 * scale)) / 2.0;
    let y_diff = (vp.height() as f32 - (tq.height as f32 * scale)) / 2.0;
    let x = (x_diff - x_diff * pan_x) as i32;
    let y = (y_diff - y_diff * pan_y) as i32;
    let width = (tq.width as f32 * scale) as u32;
    let height = (tq.height as f32 * scale) as u32;
    Rect::new(x, y, width, height)
}

/// Compute increment of skips
/// Does not account for overflow or underflow of vector
fn compute_skip_size(images: &[PathBuf]) -> usize {
    let chunks = 10usize;
    let skip_size: usize = (images.len() as usize / chunks) as usize + 1usize;

    // Skip increment must be at least 1
    cmp::max(1usize, skip_size)
}
