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
use crate::ui::{self, Action, Mode, PanAction, ProcessAction, RotationDirection, ZoomAction};
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
        let base_dir = args.base_dir;

        let max_viewable = max_length;

        let sorter = Sorter::new(sort_order, reverse);
        sorter.sort(&mut images);

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

    /// Flip image vertically
    fn flip_vertical(&mut self) -> Result<(), String> {
        self.ui_state.flip_vertical = !self.ui_state.flip_vertical;
        self.render_screen(false)?;
        Ok(())
    }

    /// Flip image horizontally
    fn flip_horizontal(&mut self) -> Result<(), String> {
        self.ui_state.flip_horizontal = !self.ui_state.flip_horizontal;
        self.render_screen(false)?;
        Ok(())
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
    pub fn skip_forward(&mut self, times: usize) -> Result<(), String> {
        let skip_size = compute_skip_size(self.paths.images());
        self.increment(skip_size * times)
    }

    /// Skips backward by the default skip increment and renders the image
    fn skip_backward(&mut self, times: usize) -> Result<(), String> {
        let skip_size = compute_skip_size(self.paths.images());
        self.decrement(skip_size * times)
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
        self.ui_state.zoom_in(times);
        self.render_screen(false)
    }

    /// Zooms out
    fn zoom_out(&mut self, times: usize) -> Result<(), String> {
        self.ui_state.zoom_out(times);
        self.render_screen(false)
    }

    /// Pans left
    fn pan_left(&mut self, times: usize) -> Result<(), String> {
        let step = self.calc_x_step();
        self.ui_state.pan_x += step * times as f32;
        if self.ui_state.pan_x > 1.0 {
            self.ui_state.pan_x = 1.0;
        }
        self.render_screen(false)
    }

    /// Pans right
    fn pan_right(&mut self, times: usize) -> Result<(), String> {
        let step = self.calc_x_step();
        self.ui_state.pan_x -= step * times as f32;
        if self.ui_state.pan_x < -1.0 {
            self.ui_state.pan_x = -1.0;
        }
        self.render_screen(false)
    }

    /// Pans up
    fn pan_up(&mut self, times: usize) -> Result<(), String> {
        let step = self.calc_y_step();
        self.ui_state.pan_y += step * times as f32;
        if self.ui_state.pan_y > 1.0 {
            self.ui_state.pan_y = 1.0;
        }
        self.render_screen(false)
    }

    /// Pans down
    fn pan_down(&mut self, times: usize) -> Result<(), String> {
        let step = self.calc_y_step();
        self.ui_state.pan_y -= step * times as f32;
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

        // Store errors for possible future use
        let mut failures: Vec<String> = Vec::new();
        for imagepath in paths {
            let newname = match self.construct_dest_filepath(imagepath) {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("{}", e);
                    failures.push(e);
                    continue;
                }
            };

            let opt = &fs_extra::file::CopyOptions::new();
            if let Err(e) = copy(imagepath, newname, opt).map_err(|e| e.to_string()) {
                eprintln!("{}", e);
                failures.push(e);
                continue;
            }
        }
        if failures.is_empty() {
            Ok(format!(
                "copied {} image(s) to {} succesfully",
                paths.len(),
                self.paths.dest_folder.to_str().unwrap(),
            ))
        } else {
            Err(format!(
                "Failed to copy {} of {} images",
                failures.len(),
                paths.len()
            ))
        }
    }

    /// Moves the current image and (n-1) next images
    /// Does nothing if supplied 0 for an amount
    fn move_images(&mut self, amount: usize) -> Result<String, String> {
        if amount == 0 {
            return Ok("0 images asked to move".to_string());
        }

        let current_index = match self.paths.index() {
            Some(i) => i,
            None => return Err("no images to move".to_string()),
        };

        // Safe to unwrap as max_index is always present if index is present
        let max_index = self.paths.max_viewable_index().unwrap();

        // Compute actual number of images to remove
        // Cap at max index. Add 1 incase max index == current_index
        let total_removes =
            std::cmp::min(current_index + amount - 1, max_index) - current_index + 1;
        // Store errors for possible future use
        let mut failures: Vec<String> = Vec::new();
        for _ in 0..total_removes {
            let current_path = self.paths.current_image_path().unwrap();
            let newname = self.construct_dest_filepath(current_path)?;
            let opt = &fs_extra::file::CopyOptions::new();

            // Attempt to move as many images as possible
            if let Err(e) = move_file(current_path, &newname, opt) {
                eprintln!("{}", e);
                failures.push(e.to_string());
                continue;
            }
            // Only if successful, remove image from tracked images
            self.paths.remove_current_image();
        }

        // Moving the image automatically advanced to next image
        // Adjust our view to reflect this
        self.screen.dirty = true;
        self.render_screen(false)?;
        if failures.is_empty() {
            let success_msg = format!(
                "moved {} image(s) succesfully to {}",
                total_removes,
                self.paths.dest_folder.to_str().unwrap()
            );
            Ok(success_msg)
        } else {
            Err(format!(
                "Failed to move {} of {} images",
                failures.len(),
                total_removes,
            ))
        }
    }

    /// Trashes image currently being viewed
    /// Does nothing if supplied 0 for an amount
    fn trash_images(&mut self, amount: usize) -> Result<String, String> {
        if amount == 0 {
            return Ok("0 images asked to trash".to_string());
        }

        let current_index = match self.paths.index() {
            Some(i) => i,
            None => return Err("no images to trash".to_string()),
        };

        let max_index = self.paths.max_viewable_index().unwrap();

        // Compute actual number of images to trash
        // Cap at max index. Add 1 incase max index == current_index
        let total_trashes =
            std::cmp::min(current_index + amount - 1, max_index) - current_index + 1;

        // Store errors for possible future use

        let mut failures: Vec<String> = Vec::new();
        // Attempt to trash as many images as possible;
        for _ in 0..total_trashes {
            let current_path = self.paths.current_image_path().unwrap();

            #[cfg(target_os = "windows")]
            {
                // Convert to msdos path to work with `move_to_trash_win`
                // Unsure why UNC paths don't work.
                let msdos_path = dunce::canonicalize(&path).unwrap();
                let trash_result = move_to_trash_win(&msdos_path);

                if let Err(e) = trash_result {
                    eprintln!("{}", e);
                    failures.push(e.to_string());
                    continue;
                }
            }
            #[cfg(target_os = "linux")]
            {
                let trash_result = trash::move_to_trash(&current_path);
                if let Err(e) = trash_result {
                    eprintln!("{}", e);
                    failures.push(e.to_string());
                    continue;
                }
            }
            #[cfg(target_os = "macos")]
            {
                // use homebrew `trash -F` command for OSX trash support
                use std::process::Command;
                let output = Command::new("trash")
                    .arg("-F")
                    .arg(&current_path)
                    .output()
                    .expect("Failed to spawn trash program");
                if !output.status.success() {
                    eprintln!("{:?}", &output);
                    failures.push(format!("{:?}: {:?}", output.status, output.stderr));
                    continue;
                }
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            return Err("Trash support for OS not supported".to_string());

            // Only if successful, remove image from tracked images
            self.paths.remove_current_image();
        }

        // Trashing the image automatically advanced to next image
        // Adjust our view to reflect this
        self.screen.dirty = true;
        self.render_screen(false)?;
        if failures.is_empty() {
            let success_msg = format!("Trashed {} image(s)", total_trashes);
            Ok(success_msg)
        } else {
            Err(format!(
                "Failed to trash {} of {} images",
                failures.len(),
                total_trashes,
            ))
        }
    }

    /// Deletes image currently being viewed
    /// Does nothing if supplied 0 for an amount
    fn delete_images(&mut self, amount: usize) -> Result<String, String> {
        if amount == 0 {
            return Ok("0 images asked to delete".to_string());
        }

        let current_index = match self.paths.index() {
            Some(i) => i,
            None => return Err("no images to delete".to_string()),
        };

        let max_index = self.paths.max_viewable_index().unwrap();

        // Compute actual number of images to remove
        // Cap at max index. Add 1 incase max index == current_index
        let total_removes =
            std::cmp::min(current_index + amount - 1, max_index) - current_index + 1;

        // Store errors for possible future use
        let mut failures: Vec<String> = Vec::new();
        // Attempt to delete as many images as possible
        for _ in 0..total_removes {
            let current_path = self.paths.current_image_path().unwrap();
            if let Err(e) = remove(current_path) {
                eprintln!("{}", e);
                failures.push(e.to_string());
                continue;
            }
            // Only if successful, remove image from tracked images
            self.paths.remove_current_image();
        }

        // Deletes the image automatically advanced to next image
        // Adjust our view to reflect this
        self.screen.dirty = true;
        self.render_screen(false)?;
        if failures.is_empty() {
            let success_msg = format!("Deleted {} image(s)", total_removes);
            Ok(success_msg)
        } else {
            Err(format!(
                "Failed to delete {} of {} images",
                failures.len(),
                total_removes,
            ))
        }
    }

    /// Jumps to specific image
    /// Caps at artificial length or last image if index supplied is too large
    fn jump_to_image_index(&mut self, index: usize) -> Result<(), String> {
        self.paths.set_index_safe(index);
        Ok(())
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
    /// Previous input from normal mode is in `self.ui_state.repeat`
    fn run_multi_normal_mode(&mut self) -> Result<(), String> {
        use ui::MultiNormalAction;

        let mut complete_action = false;
        while !complete_action {
            for event in self.screen.sdl_context.event_pump()?.poll_iter() {
                let multi_action = ui::process_multi_normal_mode(&mut self.ui_state, &event);
                // Assume input is finished unless set by other actions
                complete_action = true;

                match multi_action {
                    MultiNormalAction::MoreInput => {
                        complete_action = false;
                        // Reflect new amount to repeat on display
                        self.render_screen(false)?;
                    }
                    MultiNormalAction::Cancel => {
                        self.ui_state.mode = Mode::Normal;
                    }
                    MultiNormalAction::Repeat(process) => {
                        self.ui_state.process_action(process.clone());
                        match process {
                            ProcessAction {
                                action: a,
                                times: n,
                            } => match (a, n) {
                                (Action::Backspace, _) => {
                                    // Divide repeat by int 10 to remove last digit
                                    self.ui_state.register.cur_action.times /= 10;
                                    complete_action = false;
                                    self.render_screen(false)?;
                                    continue;
                                }
                                (Action::Last, _) => {
                                    let requested_index =
                                        self.ui_state.register.cur_action.times - 1;
                                    self.jump_to_image_index(requested_index)?;
                                    self.ui_state.mode = Mode::Normal;
                                    self.render_screen(false)?;
                                }
                                (a, _) => {
                                    self.ui_state.register.cur_action.action = a;
                                    self.ui_state.mode = Mode::Normal;
                                }
                            },
                        }
                    }
                    MultiNormalAction::SwitchBackNormalMode => {
                        self.ui_state.mode = Mode::Normal;
                    }
                    MultiNormalAction::Quit => {
                        self.ui_state.mode = Mode::Exit;
                    }
                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
        Ok(())
    }

    /// Processes Normal Mode Actions
    /// Ok result tells whether to continue or break out of the current Mode
    // Allow cognitive complexity lint since we need to match every Action
    // Note: complexity could be simplified by splittng out Command mode and Normal mode actions
    #[allow(clippy::cognitive_complexity)]
    fn dispatch_normal(&mut self, process_action: ProcessAction) -> Result<CompleteType, String> {
        match process_action {
            ProcessAction { action, times } => match action {
                Action::Quit => {
                    self.ui_state.mode = Mode::Exit;
                    return Ok(CompleteType::Break);
                }
                Action::ToggleFullscreen => {
                    self.toggle_fullscreen();
                    self.screen.update_fullscreen(self.ui_state.fullscreen)?;
                    self.render_screen(false)?
                }
                Action::ReRender => self.render_screen(false)?,
                Action::SwitchCommandMode => {
                    self.ui_state.mode = Mode::Command(String::new());
                    return Ok(CompleteType::Break);
                }
                Action::SwitchMultiNormalMode => {
                    self.ui_state.mode = Mode::MultiNormal;
                    return Ok(CompleteType::Break);
                }
                Action::ToggleFit => self.toggle_fit()?,
                Action::FlipHorizontal => self.flip_horizontal()?,
                Action::FlipVertical => self.flip_vertical()?,
                Action::CenterImage => self.center_image()?,
                Action::Next => self.increment(times)?,
                Action::Prev => self.decrement(times)?,
                Action::First => self.first()?,
                Action::Last => self.last()?,
                Action::SkipForward => self.skip_forward(times)?,
                Action::SkipBack => self.skip_backward(times)?,
                Action::Zoom(ZoomAction::In) => self.zoom_in(times)?,
                Action::Zoom(ZoomAction::Out) => self.zoom_out(times)?,
                Action::Rotate(RotationDirection::Clockwise) => {
                    self.ui_state.rot_angle = self.ui_state.rot_angle.rot_clockwise();
                    self.render_screen(false)?;
                }
                Action::Rotate(RotationDirection::CounterClockwise) => {
                    self.ui_state.rot_angle = self.ui_state.rot_angle.rot_clockclockwise();
                    self.render_screen(false)?;
                }
                Action::Pan(PanAction::Left) => self.pan_left(times)?,
                Action::Pan(PanAction::Right) => self.pan_right(times)?,
                Action::Pan(PanAction::Up) => self.pan_up(times)?,
                Action::Pan(PanAction::Down) => self.pan_down(times)?,
                Action::Copy => match self.copy_images(times) {
                    Ok(s) => {
                        self.ui_state.mode = Mode::Success(s);
                        self.ui_state.rerender_time = Some(Instant::now());
                        return Ok(CompleteType::Break);
                    }
                    Err(e) => {
                        self.ui_state.mode = Mode::Error(format!("Failed to copy file: {}", e));
                        return Ok(CompleteType::Break);
                    }
                },
                Action::Move => match self.move_images(times) {
                    Ok(s) => {
                        self.ui_state.mode = Mode::Success(s);
                        self.ui_state.rerender_time = Some(Instant::now());
                        return Ok(CompleteType::Break);
                    }
                    Err(e) => {
                        self.ui_state.mode = Mode::Error(format!("Failed to move file: {}", e));
                        return Ok(CompleteType::Break);
                    }
                },
                Action::Delete => match self.delete_images(times) {
                    Ok(s) => {
                        self.ui_state.mode = Mode::Success(s);
                        self.ui_state.rerender_time = Some(Instant::now());
                        return Ok(CompleteType::Break);
                    }
                    Err(e) => {
                        self.ui_state.mode = Mode::Error(format!("Failed to delete file: {}", e));
                        return Ok(CompleteType::Break);
                    }
                },
                Action::Trash => match self.trash_images(times) {
                    Ok(s) => {
                        self.ui_state.mode = Mode::Success(s);
                        self.ui_state.rerender_time = Some(Instant::now());
                        return Ok(CompleteType::Break);
                    }
                    Err(e) => {
                        self.ui_state.mode = Mode::Error(format!("Failed to delete file: {}", e));
                        return Ok(CompleteType::Break);
                    }
                },
                Action::Noop => return Ok(CompleteType::Complete),
                _ => return Ok(CompleteType::Complete),
            },
        }

        Ok(CompleteType::Complete)
    }

    /// run_normal_mode is the event loop that listens for input and delegates accordingly for
    /// normal mode
    fn run_normal_mode(&mut self) -> Result<(), String> {
        'mainloop: loop {
            for event in self.screen.sdl_context.event_pump()?.poll_iter() {
                let action = ui::process_normal_mode(&mut self.ui_state, &event);
                self.ui_state.process_action(action.clone());
                let next_step = self.dispatch_normal(action.clone())?;
                if let CompleteType::Break = next_step {
                    break 'mainloop;
                }

                // Were we just in multiMode
                match self.ui_state.register.cur_action {
                    ProcessAction {
                        action: Action::Noop,
                        ..
                    } => {}
                    _ => {
                        // Process the action we need to perform in bulk
                        let next_step =
                            self.dispatch_normal(self.ui_state.register.cur_action.clone())?;
                        // Clear out stored action for next bulk action
                        self.ui_state.register.cur_action = ProcessAction::default();

                        // Reset repeat register to 1 after performing the action i bulk
                        //self.ui_state.register.cur_action.times = 1;

                        // Switch modes if action needs to modify the UI
                        if let CompleteType::Break = next_step {
                            break 'mainloop;
                        }
                    }
                }
            }

            if let Some(ts) = self.ui_state.rerender_time {
                if Instant::now().duration_since(ts) > Duration::from_millis(1500) {
                    self.ui_state.rerender_time = None;
                    self.ui_state.mode = Mode::Normal;
                    return Ok(());
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

enum CompleteType {
    Complete,
    Break,
}

#[cfg(target_os = "windows")]
/// Moves a file to the trash on Windows
fn move_to_trash_win<S: AsRef<OsStr>>(file: S) -> Result<i32, i32> {
    use std::ffi::OsStr;
    use std::iter::repeat;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::{SHFileOperationW, SHFILEOPSTRUCTW};

    use std::ptr::{null, null_mut};
    use winapi::um::shellapi;

    // Encode the file as wide or UTF-16.
    // Double null terminate end of filename for PCZZSTR type
    let wide: Vec<u16> = OsStr::new(file.as_ref())
        .encode_wide()
        .chain(repeat(0).take(2))
        .collect();

    let mut trashed = SHFILEOPSTRUCTW {
        hwnd: null_mut(),
        wFunc: u32::from(shellapi::FO_DELETE),
        fFlags: shellapi::FOF_ALLOWUNDO | shellapi::FOF_NO_UI,
        fAnyOperationsAborted: 0,
        pFrom: wide.as_ptr(),
        pTo: null(),
        hNameMappings: null_mut(),
        lpszProgressTitle: null_mut(),
    };

    let ret = unsafe { SHFileOperationW(&mut trashed) };
    // 0 return code is success. Anything else is an error
    if ret == 0 {
        Ok(ret)
    } else {
        Err(ret)
    }
}
