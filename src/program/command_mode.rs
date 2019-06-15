//! File that contains Command mode functionality, command mode is a mode that allows verbose input
//! from the user to perform tasks or edit stored data in the application during runtime
use crate::infobar::Text;
use crate::paths::{incremental_glob, SendStatus};
use crate::program::{mode_colors, mode_text_color, Program};
use crate::screen::Screen;
use crate::sort::SortOrder;
use crate::ui::{process_command_mode, Action, HelpRender, Mode};
use crossbeam_channel::bounded;
use crossbeam_utils::thread;

use regex::Regex;
use shellexpand::full;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Available commands in Command mode
///
/// Note: in documentation for commands leading `:` is prepended and should not be included
enum Commands {
    /// `:sort`
    ///
    /// No argument: performs the selected sort on images.
    /// One argument: argument is the new sorting order to perform, subsequent calls to sort performs newly
    /// selected sort.
    ///
    /// Regardless of arguments, if the current image prior to sort is in the post sorted images
    /// move to its index
    Sort,
    /// `:ng` or `:newglob`
    ///
    /// Requires only one addition parameter, the new current_dir.
    /// If the current image exists prior to changing globs exists in the new glob move to that index.
    /// If the new path has no images do nothing.
    NewGlob,
    /// `:?` or `:help`
    ///
    /// Switches to normal mode and displays help info
    Help,
    /// `:q` or `:quit`
    ///
    /// Terminates the application
    Quit,
    /// `:r` or `:reverse`
    ///
    /// reverses current images, moving to index of current image prior to reverse
    Reverse,
    /// `:df` or `:destfolder`
    ///
    /// Requires one argument, the new path for the destination folder (where to save images)
    DestFolder,
    /// `:m` or `:max`
    ///
    /// Sets the maximum number of images to display at any given time
    MaximumImages,
}

impl FromStr for Commands {
    type Err = String;

    /// All commands must implement FromStr
    fn from_str(s: &str) -> Result<Commands, String> {
        match s {
            "sort" => Ok(Commands::Sort),
            "ng" | "newglob" => Ok(Commands::NewGlob),
            "?" | "help" => Ok(Commands::Help),
            "q" | "quit" => Ok(Commands::Quit),
            "r" | "reverse" => Ok(Commands::Reverse),
            "df" | "destfolder" => Ok(Commands::DestFolder),
            "m" | "max" => Ok(Commands::MaximumImages),
            _ => Err(format!(
                "No such command \"{}\", type :? for command help",
                s
            )),
        }
    }
}

/// Globs the passed path, returning an error if no images are in that path, glob::glob fails, or
/// path is unexpected
fn glob_path(screen: &mut Screen, path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let glob = glob::glob(&path.to_string_lossy()).map_err(|e| e.to_string())?;
    let (tx, rx) = bounded(5);
    let mut new_images = Vec::new();

    let theme = mode_colors(&Mode::Command("".to_string()));
    let text_color = mode_text_color(&Mode::Command("".to_string()));

    thread::scope(|scope| {
        scope.spawn(|_| incremental_glob(glob, &tx, &mut new_images));
        // loop till scan is complete
        loop {
            match rx.recv() {
                Ok(SendStatus::Started) => {
                    let text = Text {
                        child_1: " ".to_string(),
                        child_2: "Starting to scan images".to_string(),
                    };
                    screen.render_infobar(text, text_color, &theme).unwrap();
                    screen.canvas.present()
                }
                Ok(SendStatus::Progress(n)) => {
                    let text = Text {
                        child_1: "In progress".to_string(),
                        child_2: format!("matched {} images", n),
                    };
                    screen.render_infobar(text, text_color, &theme).unwrap();
                    screen.canvas.present();
                }
                Ok(SendStatus::Complete(_)) => {
                    break;
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        }
    })
    .unwrap();
    if new_images.is_empty() {
        let err_msg = format!("Path \"{}\" had no images", path.display());
        return Err(err_msg);
    }
    Ok(new_images)
}

/// Separate user input into the main command and its respected arguments
fn parse_user_input(input: String) -> Result<(Commands, String), String> {
    // find where to split
    let command_terminating_index = {
        if let Some(space_index) = input.find(' ') {
            space_index
        } else {
            input.len()
        }
    };
    let command_str = &input[0..command_terminating_index];
    let command = Commands::from_str(command_str)?;
    let arguments = {
        if input.len() > command_terminating_index {
            input[command_terminating_index + 1..].to_owned()
        } else {
            String::new()
        }
    };
    Ok((command, arguments))
}

impl<'a> Program<'a> {
    /// User input is taken in and displayed on infobar, cmd is either '/' or ':'
    /// Returning empty string signifies switching modes back to normal mode
    fn get_command(&mut self, cmd: &str) -> Result<String, String> {
        let mut input = String::new();
        let mut events = self.screen.sdl_context.event_pump()?;
        'command_loop: loop {
            for event in events.poll_iter() {
                let action = process_command_mode(&event);
                match action {
                    Action::Backspace => {
                        if input.is_empty() {
                            break 'command_loop;
                        }
                        input.pop();
                        self.ui_state.mode = Mode::Command(input.clone());
                        self.render_screen(false)?;
                    }
                    Action::KeyboardInput(text) => {
                        input.push_str(text);
                        // Fixes additional ':' in command mode start
                        if input.starts_with(cmd) {
                            input = input[1..].to_string();
                        }
                        self.ui_state.mode = Mode::Command(input.clone());
                        self.render_screen(false)?;
                    }
                    Action::SwitchNormalMode => break 'command_loop,
                    _ => continue,
                }
            }
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
        Ok(input)
    }

    /// Takes a path to a directory or glob and adds these images to self.paths.images
    fn newglob(&mut self, path_to_newglob: &str) {
        let path = match crate::path_to_glob(&self.paths.base_dir, path_to_newglob) {
            Ok(path) => path,
            Err(e) => {
                self.ui_state.mode = Mode::Error(e.to_string());
                return;
            }
        };
        let msg = path_to_newglob.to_owned();

        let new_images = match glob_path(&mut self.screen, &path) {
            Ok(new_images) => new_images,
            Err(e) => {
                self.ui_state.mode = Mode::Error(e.to_string());
                return;
            }
        };

        let target = match self.paths.current_image_path() {
            Some(path) => {
                // Clone the path because the whole image set is going to be swapped out
                Some(path.clone())
            }
            // Anything else, we are dealing with no images
            None => None,
        };

        self.paths.reload_images(new_images);

        // Set current directory to new one
        let new_base_dir = crate::new_base_dir(&path);
        if let Ok(base_dir) = new_base_dir {
            self.paths.base_dir = base_dir
        }
        self.sorter.sort(self.paths.images_as_mut_slice());
        if let Some(target_path) = target {
            if let Some(new_index) = self
                .paths
                .images()
                .iter()
                .position(|path| path == &target_path)
            {
                if let Some(max_i) = self.paths.max_viewable_index() {
                    if new_index > max_i {
                        self.paths.set_index(0);
                    } else {
                        self.paths.set_index(new_index);
                    }
                }
            }
        }
        self.ui_state.mode = Mode::Success(format!(
            "found {} images in {}",
            self.paths.images().len(),
            msg
        ));
        self.ui_state.rerender_time = Some(Instant::now());
    }

    /// Providing no additional arguments just sorts the current images with the already set sorting
    /// method
    ///
    /// Additional argument changes the sorting method and sorts the images
    fn sort(&mut self, arguments: String) {
        if arguments.is_empty() {
            self.sorter.sort(self.paths.images_as_mut_slice());
            return;
        }
        // get a SortOrder from the provided argument
        let new_sort_order = match SortOrder::from_str(&arguments) {
            Ok(order) => order,
            Err(e) => {
                self.ui_state.mode =
                    Mode::Command(format!("Invalid value \"{}\". {}", arguments, e));
                return;
            }
        };
        self.sorter.set_order(new_sort_order);

        self.sorter.sort(self.paths.images_as_mut_slice());

        // the path to find in order to maintain that it is the current image
        let (target_path, max_index) = match (
            self.paths.current_image_path(),
            self.paths.max_viewable_index(),
        ) {
            (Some(path), Some(index)) => (path, index),
            // Anything else, we are dealing with no images
            (_, _) => return,
        };

        // We know there is at least 1 image present
        let new_index = self
            .paths
            .images()
            .iter()
            .position(|path| path == target_path)
            // Safe to unwrap as we just got the target path above
            .unwrap();

        if new_index <= max_index {
            self.paths.set_index(new_index);
        } else {
            self.paths.set_index(0);
        }
    }

    /// sets the new maximum_viewable images
    fn maximum_viewable(&mut self, max: &str) {
        let new_actual_max = match max.parse::<usize>() {
            Ok(new_max) => new_max,
            Err(_e) => {
                self.ui_state.mode = Mode::Error(format!("\"{}\" is not a positive integer", max));
                return;
            }
        };
        self.paths.set_actual_maximum(new_actual_max);
    }

    /// Enters command mode that gets user input and runs a set of possible commands based on user input.
    /// After every command the user is set either into normal mode or the app terminates.
    ///
    /// List of commands provided in `Commands` enum
    ///
    /// Error is returned only in serious cases, for instance if the application fails to render_screen
    pub fn run_command_mode(&mut self) -> Result<(), String> {
        self.ui_state.render_infobar = true;
        self.render_screen(false)?;
        let input = self.get_command(":")?;
        // after evaluating a command always exit to normal mode by default
        self.ui_state.mode = Mode::Normal;
        // Empty input means switch back to normal mode
        if input.is_empty() {
            return Ok(());
        }
        let (command, arguments) = match parse_user_input(input) {
            Ok((command, arguments)) => (command, arguments),
            Err(e) => {
                self.ui_state.mode = Mode::Error(e.to_string());
                return Ok(());
            }
        };
        match command {
            Commands::NewGlob => {
                if arguments.is_empty() {
                    self.ui_state.mode =
                        Mode::Error(("Command \"newglob\" or \":ng\" requires a glob").to_string());
                    return Ok(());
                }
                self.newglob(&arguments);
            }
            Commands::Help => match self.ui_state.render_help {
                HelpRender::Command => self.ui_state.render_help = HelpRender::None,
                _ => self.ui_state.render_help = HelpRender::Command,
            },
            Commands::Quit => {
                self.ui_state.mode = Mode::Exit;
            }
            Commands::Reverse => {
                self.paths.reverse();
            }
            Commands::DestFolder => {
                if arguments.is_empty() {
                    self.ui_state.mode = Mode::Error(
                        "Command \":destfolder\" or \":d\" requires a path".to_string(),
                    );
                    return Ok(());
                }
                match full(&arguments) {
                    Ok(path) => {
                        let mut path = path.to_string();
                        if cfg!(unix) {
                            lazy_static! {
                                static ref REGEX_REMOVE_ESCAPED_CHARS: Regex =
                                    match Regex::new(r"\\(.)") {
                                        Ok(regex) => regex,
                                        Err(e) => panic!("Logic Error: {}", e),
                                    };
                            }
                            path = REGEX_REMOVE_ESCAPED_CHARS
                                .replace_all(&path, "$1")
                                .to_string();
                        }
                        let success_msg =
                            format!("destination folder successfully set to {}", path);
                        self.paths.dest_folder = PathBuf::from(path);
                        self.ui_state.mode = Mode::Success(success_msg);
                        self.ui_state.rerender_time = Some(Instant::now());
                    }
                    Err(e) => {
                        self.ui_state.mode =
                            Mode::Error(format!("\"{}\": {}", e.var_name, e.cause));
                        return Ok(());
                    }
                }
            }
            Commands::MaximumImages => {
                if arguments.is_empty() {
                    self.ui_state.mode = Mode::Error(
                        "Command \":max\" or \":m\" requires a new maximum number of files to display".to_string(),
                    );
                    return Ok(());
                }
                self.maximum_viewable(&arguments);
            }
            Commands::Sort => {
                self.sort(arguments);
            }
        }
        Ok(())
    }
}
