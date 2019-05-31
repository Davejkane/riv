//! # InfoBar
//!
//! Module InfoBar provides structures and functions for building and rendering an infobar

use crate::paths::Paths;
use crate::ui::{Mode, State};

/// Text contains the strings required to print the infobar.
pub struct Text {
    /// Either displays the name of the current image or the current command the user is typing in
    /// command mode
    pub information: String,
    /// In normal mode this is the string represention of the index, in command mode this is
    /// "Command"
    pub mode: String,
}

impl Text {
    /// Updates the infobar based on the current mode of the applicaiton
    /// Normal Mode:
    ///     mode = index of current image
    ///     information = path to current image
    /// Command Mode:
    ///     mode = "Command"
    ///     information = curerntly entered user string
    /// Error Mode:
    ///     mode = "Error"
    ///     information = error message to display
    pub fn update(current_mode: &Mode, paths: &Paths, state: &State) -> Self {
        let (mode, information) = match current_mode {
            Mode::Command(msg) => ("Command".to_string(), format!(":{}", msg)),
            Mode::Normal => {
                let information = match paths.current_image_path() {
                    Some(path) => match path.to_str() {
                        Some(name) => name.to_string(),
                        None => "No file".to_string(),
                    },
                    None => "No file selected".to_string(),
                };

                let mode = match paths.current_image() {
                    Some(current) => format!("{} of {}", current, paths.max_viewable().unwrap()),
                    None => "No files in path".to_string(),
                };

                (mode, information)
            }
            Mode::MultiNormal => {
                let times = state.repeat;
                (times.to_string(), " ".to_string())
            }
            Mode::Error(msg) => ("Error".to_string(), msg.to_string()),
            Mode::Success(msg) => ("Success".to_string(), msg.to_string()),
            Mode::Exit => ("Exit".to_string(), "Exiting... Goodbye".to_string()),
        };
        Text { information, mode }
    }
}
