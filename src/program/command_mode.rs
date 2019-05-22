//! File that contains Command mode functionality, command mode is a mode that allows verbose input
//! from the user to perform tasks or edit stored data in the application during runtime
use super::Program;
use crate::paths::push_image_path;
use crate::sort::SortOrder;
use crate::ui::{process_command_mode, Action, Mode};
use shellexpand::full;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

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
            _ => Err(format!("No such command \"{}\"", s)),
        }
    }
}

// Sanitize path by substituting env vars and removing escaped spaces
fn sanitize_path(path: &str, base_dir: &PathBuf) -> Result<PathBuf, String> {
    let expanded_path = full(path).map_err(|e| format!("\"{}\": {}", e.var_name, e.cause))?;
    // remove escaped spaces
    let non_escaped_path = String::from(expanded_path).replace(r"\ ", " ");
    let mut path = PathBuf::from(&non_escaped_path);
    if path.is_relative() {
        path = base_dir.join(&path);
    }
    Ok(path)
}

/// Returns a Vec of paths from globbing the path provided, a new base_dir by removing environment variables and normalizing
/// the path
fn glob_path(path: &str, base_dir: &PathBuf) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let mut new_images: Vec<PathBuf> = Vec::new();

    // remove env vars and "\ "
    let sanitized_dir = sanitize_path(path, base_dir)?;
    // normalized path removing ".." and "."
    let base_dir = sanitized_dir.canonicalize().map_err(|e| e.to_string())?;

    // Convert base_dir to a glob
    let globable_path = {
        let mut globable_path = base_dir.clone();
        if globable_path.is_dir() {
            globable_path.push("*");
        }
        globable_path.to_string_lossy().to_string()
    };
    // glob path
    let path_matches = glob::glob(&globable_path).map_err(|e| e.to_string())?;
    for path in path_matches {
        match path {
            Ok(p) => {
                push_image_path(&mut new_images, p);
            }
            Err(e) => {
                let err_msg = format!("Unexpected path {}", e);
                return Err(err_msg);
            }
        }
    }
    if new_images.is_empty() {
        let err_msg = format!("Path \"{}\" had no images", path);
        return Err(err_msg);
    }
    Ok((base_dir, new_images))
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
        let (base_dir, new_images) = match glob_path(path_to_newglob, &self.paths.base_dir) {
            Ok(new_images) => new_images,
            Err(e) => {
                self.ui_state.mode = Mode::Error(e.to_string());
                return;
            }
        };
        let target = if !self.paths.images.is_empty() {
            Some(self.paths.images[self.paths.index].to_owned())
        } else {
            None
        };
        self.paths.images = new_images;
        // Set base dir to new directory
        self.paths.base_dir = base_dir;
        self.sorter.sort(&mut self.paths.images);

        if let Some(target_path) = target {
            // find location of current image, if it exists in self.paths.images
            match self
                .paths
                .images
                .iter()
                .position(|path| path == &target_path)
            {
                Some(new_index) => self.paths.index = new_index,
                None => {
                    self.paths.index = 0;
                }
            }
        }
        self.paths.max_viewable = if self.paths.actual_max_viewable > 0
            && self.paths.actual_max_viewable <= self.paths.images.len()
        {
            self.paths.actual_max_viewable
        } else {
            self.paths.images.len()
        };
    }

    /// Providing no additional arguments just sorts the current images with the already set sorting
    /// method
    ///
    /// Additional argument changes the sorting method and sorts the images
    fn sort(&mut self, arguments: String) {
        if arguments.is_empty() {
            self.sorter.sort(&mut self.paths.images);
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
        // the path to find in order to maintain that it is the current image
        let target = if !self.paths.images.is_empty() {
            Some(self.paths.images[self.paths.index].to_owned())
        } else {
            None
        };
        self.sorter.sort(&mut self.paths.images);
        if let Some(target_path) = target {
            // find location of current image, if it exists in self.paths.images
            match self
                .paths
                .images
                .iter()
                .position(|path| path == &target_path)
            {
                Some(new_index) => {
                    if new_index <= (self.paths.max_viewable - 1) {
                        self.paths.index = new_index;
                    } else {
                        self.paths.index = 0;
                    }
                }
                None => {
                    self.paths.index = 0;
                }
            }
        }
    }

    /// sets the new maximum_viewable images
    fn maximum_viewable(&mut self, max: &str) {
        self.paths.actual_max_viewable = match max.parse::<usize>() {
            Ok(new_max) => new_max,
            Err(_e) => {
                self.ui_state.mode = Mode::Error(format!("\"{}\" is not a positive integer", max));
                return;
            }
        };
        if self.paths.actual_max_viewable > self.paths.images.len()
            || self.paths.actual_max_viewable == 0
        {
            self.paths.max_viewable = self.paths.images.len();
        } else {
            self.paths.max_viewable = self.paths.actual_max_viewable;
        }
        if self.paths.max_viewable <= self.paths.index {
            self.paths.index = self.paths.max_viewable - 1;
        }
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
            Commands::Help => {
                self.ui_state.render_help = !self.ui_state.render_help;
            }
            Commands::Quit => {
                self.ui_state.mode = Mode::Exit;
            }
            Commands::Reverse => {
                self.paths.images.reverse();
                self.paths.index = self.paths.max_viewable - self.paths.index - 1;
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
                        self.paths.dest_folder =
                            PathBuf::from(path.to_string().replace("\\ ", " "));
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
