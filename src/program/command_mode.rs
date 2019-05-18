//! File that contains Command mode functionality, command mode in this case dictates anything that
//! queries the user for input during run time
use super::Program;
use crate::sort::SortOrder;
use crate::ui::{process_command_mode, Action, Mode};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

/// Finds the provided path in paths returning the usize or error
/// Returns 0 if not found
fn find_path_in_paths(paths: &Vec<PathBuf>, current_path: &PathBuf) -> Option<usize> {
    match paths.iter().position(|path| path == current_path) {
        Some(i) => Some(i),
        None => None,
    }
}

/// Converts the provided path by user to a path that can be glob'd
/// Changes $HOME and ~ to their expanded home path
/// If the path is to a directory add glob to the end to catch the directories files
fn convert_path_to_globable(path: &str) -> Result<String, String> {
    let mut absolute_path = String::from(path);
    // Get HOME environment variable
    let home = match std::env::var("HOME") {
        Ok(home) => home,
        Err(e) => return Err(e.to_string()),
    };

    // create path_buf from path to get parents
    let mut path_buf = PathBuf::from(path);

    // ~ constant used to check if any parent is '~'
    let tilda = Path::new("~");
    // $HOME constant used to check if any parent is "$HOME"
    let home_shell = Path::new("$HOME");
    // replace all ~ and $HOME with home env variable
    for parent in path_buf.ancestors() {
        if parent == tilda {
            absolute_path = absolute_path.replace("~", &home);
        } else if parent == home_shell {
            absolute_path = absolute_path.replace("$HOME", &home);
        }
    }

    path_buf = PathBuf::from(&absolute_path);
    // If path is a dir, add /* to glob
    if path_buf.is_dir() {
        if !absolute_path.ends_with("/") {
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

impl<'a> Program<'a> {
    /// User input is taken in and displayed on infobar, cmd is either '/' or ':'
    /// Returning empty string signifies switching modes back to normal mode
    // TODO: autocomplete use fn
    fn get_command(&mut self, cmd: &str) -> Result<String, String> {
        let mut input = String::new();
        let mut events = self.screen.sdl_context.event_pump()?;
        'command_loop: loop {
            // text_input could not be stopped
            for event in events.poll_iter() {
                let action = process_command_mode(&event);

                let display = format!("{}{}", cmd, input);
                self.render_screen(false, Some(&display))?;
                match action {
                    Action::Backspace => {
                        if input.len() < 1 {
                            break 'command_loop;
                        }
                        input.pop();
                    }
                    Action::KeyboardInput(text) => {
                        input.push_str(text);
                        if input.starts_with(cmd) {
                            input = input[1..].to_string();
                        }
                    }
                    Action::SwitchNormalMode => break 'command_loop,
                    _ => continue,
                }
            }
            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
        Ok(input)
    }

    /// Enters command mode that gets user input and runs a set of possible commands based on user
    /// input. After every command the user is set either into normal mode again or the app
    /// terminates
    ///
    /// Commands:
    ///     * ng/newglob                  requires only one addition parameter, the new current_dir.
    ///                                     If the current image prior exists in the new glob move to that index
    ///
    ///     * h/help                      switches to normal mode and displays help info
    ///
    ///     * q/quit                      terminates the application
    ///
    ///     * r/reverse                   reverses current images, moving to index of current image prior to reverse
    ///
    ///     * df/destfolder               sets the new destination folder
    ///
    ///     * sort                        no argument: performs the selected sort on images, keeps
    ///                                     moves to index of current image prior to reverse
    ///                                   one argument: performs selected sort
    ///
    ///     * m/max                       set new maximum amount of images to display
    pub fn run_command_mode(&mut self) -> Result<(), String> {
        let input = self.get_command(":")?;
        // after evaluating a command always exit to normal mode by default
        self.ui_state.mode = Mode::Normal;
        // Empty input means switch back to normal mode
        if input.is_empty() {
            return Ok(());
        }
        let input_vec: Vec<&str> = input.split_whitespace().collect();

        match input_vec[0] {
            "ng" | "newglob" => {
                if input_vec.len() < 2 {
                    let err_msg =
                        String::from("Error: command \"newglob\" or \":ng\" requires a glob");
                    return Err(err_msg);
                }
                let mut new_images: Vec<PathBuf>;
                new_images = glob_path(input_vec[1])?;
                // the path to find in order to maintain that it is the current image
                let target = self.paths.images[self.paths.index].to_owned();
                self.paths.images = new_images;
                self.sorter.sort(&mut self.paths.images);
                match find_path_in_paths(&self.paths.images, &target) {
                    Some(new_index) => self.paths.index = new_index,
                    None => {
                        self.paths.index = 0;
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
            "h" | "help" => {
                self.ui_state.render_help = !self.ui_state.render_help;
            }
            "q" | "quit" => {
                self.ui_state.mode = Mode::Exit;
            }
            "r" | "reverse" => {
                self.paths.images.reverse();
                self.paths.index = self.paths.max_viewable - self.paths.index - 1;
            }
            "df" | "destfolder" => {
                if input_vec.len() < 2 {
                    let err_msg =
                        String::from("Error: command \":destfolder\" or \":d\" requires a path");
                    return Err(err_msg);
                }
                self.paths.dest_folder = PathBuf::from(input_vec[1]);
            }
            "m" | "max" => {
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
            "sort" => {
                use std::str::FromStr;

                // Allow both just calling "sort" and allow providing the new sort
                if input_vec.len() >= 2 {
                    let new_sort_order = match SortOrder::from_str(input_vec[1]) {
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
                match find_path_in_paths(&self.paths.images, &target) {
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
            _ => {
                let err_msg = format!("Error: \"{}\" is not a command", input_vec[0]);
                return Err(err_msg);
            }
        }
        Ok(())
    }
}
