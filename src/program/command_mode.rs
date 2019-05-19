//! File that contains Command mode functionality, command mode is a mode that allows verbose input
//! from the user to perform tasks or edit stored data in the application during runtime
use super::Program;
use crate::sort::SortOrder;
use crate::ui::{process_command_mode, Action, Mode};
use regex::Regex;
use shellexpand::full;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

/// Available commands in Command mode
///
/// Note: in documentation for commands leading `:` is prepended and should no be included
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
    /// `:h` or `:help`
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
            "h" | "help" => Ok(Commands::Help),
            "q" | "quit" => Ok(Commands::Quit),
            "r" | "reverse" => Ok(Commands::Reverse),
            "df" | "destfolder" => Ok(Commands::DestFolder),
            "m" | "max" => Ok(Commands::MaximumImages),
            _ => Err(format!("Error: no such command \"{}\"", s)),
        }
    }
}

/// Converts the provided path by user to a path that can be glob'd
/// Directories are changed from /home/etc to /home/etc/*
fn convert_path_to_globable(path: &str) -> Result<String, String> {
    let expanded_path =
        full(path).map_err(|e| format!("Error: \"{}\": {}", e.var_name, e.cause))?;
    let mut absolute_path = String::from(expanded_path);
    // If path is a dir, add /* to glob
    if PathBuf::from(&absolute_path).is_dir() {
        if !absolute_path.ends_with('/') {
            absolute_path.push('/');
        }
        absolute_path.push('*');
    }
    Ok(absolute_path)
}

/// Globs the passed path, returning an error if no images are in that path, glob::glob fails, or
/// path is unexpected
fn glob_path(path: &str) -> Result<Vec<PathBuf>, String> {
    use crate::cli::push_image_path;

    let mut new_images: Vec<PathBuf> = Vec::new();
    let globable_path = convert_path_to_globable(path)?;
    let path_matches = glob::glob(&globable_path).map_err(|e| e.to_string())?;
    for path in path_matches {
        match path {
            Ok(p) => {
                push_image_path(&mut new_images, p);
            }
            Err(e) => {
                let err_msg = format!("Error: Unexpected path {}", e);
                return Err(err_msg);
            }
        }
    }
    if new_images.is_empty() {
        let err_msg = format!("Error: path \"{}\" had no images", path);
        return Err(err_msg);
    }
    Ok(new_images)
}

/// Uses regex to parse user input, then concating all subsequent strings that are escaped together
/// TODO: have regex capture the ending space in case of escaped files
fn parse_user_input(input: &str) -> Vec<String> {
    let mut input_vec: Vec<String> = Vec::new();
    let re: Regex = Regex::new(r"[\x21-\x7E]+(\\)?").unwrap();
    let mut escaped = false;

    for regex_match in re.captures_iter(&input) {
        let value: String = regex_match[0].to_string();
        if value.ends_with('\\') && !escaped {
            escaped = true;
            input_vec.push(value);
        } else if escaped {
            if let Some(last) = input_vec.last_mut() {
                last.push(' ');
                last.push_str(&value);
            }
        } else {
            input_vec.push(value);
        }
    }
    input_vec
}

impl<'a> Program<'a> {
    /// User input is taken in and displayed on infobar, cmd is either '/' or ':'
    /// Returning empty string signifies switching modes back to normal mode
    // TODO: autocomplete use fn
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
                        let display = format!("{}{}", cmd, input);
                        self.render_screen(false, Some(&display))?;
                    }
                    Action::KeyboardInput(text) => {
                        input.push_str(text);
                        // Fixes additional ':' in command mode start
                        if input.starts_with(cmd) {
                            input = input[1..].to_string();
                        }
                        let display = format!("{}{}", cmd, input);
                        self.render_screen(false, Some(&display))?;
                    }
                    Action::SwitchNormalMode => break 'command_loop,
                    _ => continue,
                }
            }
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
        Ok(input)
    }

    /// Enters command mode that gets user input and runs a set of possible commands based on user input.
    /// After every command the user is set either into normal mode or the app terminates.
    ///
    /// List of commands provided in `Commands` enum
    pub fn run_command_mode(&mut self) -> Result<(), String> {
        let input = self.get_command(":")?;
        // after evaluating a command always exit to normal mode by default
        self.ui_state.mode = Mode::Normal;
        // Empty input means switch back to normal mode
        if input.is_empty() {
            return Ok(());
        }
        let input_vec = parse_user_input(&input);
        let command = Commands::from_str(&input_vec[0])?;
        match command {
            Commands::NewGlob => {
                if input_vec.len() < 2 {
                    let err_msg =
                        String::from("Error: command \"newglob\" or \":ng\" requires a glob");
                    return Err(err_msg);
                }
                let mut new_images: Vec<PathBuf>;
                new_images = glob_path(&input_vec[1])?;
                let target = if !self.paths.images.is_empty() {
                    Some(self.paths.images[self.paths.index].to_owned())
                } else {
                    None
                };
                self.paths.images = new_images;
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
                self.paths.max_viewable = if self.paths.max_viewable > 0
                    && self.paths.max_viewable <= self.paths.images.len()
                {
                    self.paths.max_viewable
                } else {
                    self.paths.images.len()
                };
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
                if input_vec.len() < 2 {
                    let err_msg =
                        String::from("Error: command \":destfolder\" or \":d\" requires a path");
                    return Err(err_msg);
                }
                self.paths.dest_folder = PathBuf::from(&input_vec[1]);
            }
            Commands::MaximumImages => {
                if input_vec.len() < 2 {
                    let err_msg =
                        String::from("Error: command \":max\" or \":m\" requires a new maximum number of files to display");
                    return Err(err_msg);
                }
                self.paths.max_viewable = match input_vec[1].parse::<usize>() {
                    Ok(new_max) => new_max,
                    Err(_e) => {
                        let err_msg =
                            format!("Error: \"{}\" is not a positive integer", input_vec[1]);
                        return Err(err_msg);
                    }
                };
                if self.paths.max_viewable > self.paths.images.len() || self.paths.max_viewable == 0
                {
                    self.paths.max_viewable = self.paths.images.len();
                }
                if self.paths.max_viewable < self.paths.index {
                    self.paths.index = self.paths.max_viewable - 1;
                }
            }
            Commands::Sort => {
                // Allow both just calling "sort" and allow providing the new sort
                if input_vec.len() >= 2 {
                    let new_sort_order = match SortOrder::from_str(&input_vec[1]) {
                        Ok(order) => order,
                        Err(e) => {
                            return Err(format!(
                                "Error: invalid value \"{}\". {}",
                                input_vec[1], e
                            ));
                        }
                    };
                    self.sorter.set_order(new_sort_order);
                }
                // the path to find in order to maintain that it is the current image
                let target = self.paths.images[self.paths.index].to_owned();
                self.sorter.sort(&mut self.paths.images);
                match self.paths.images.iter().position(|path| path == &target) {
                    Some(new_index) => {
                        if new_index < self.paths.max_viewable {
                            self.paths.index = new_index;
                        } else {
                            self.paths.index = self.paths.max_viewable - 1;
                        }
                    }
                    None => {
                        self.paths.index = 0;
                    }
                }
            }
        }
        Ok(())
    }
}
