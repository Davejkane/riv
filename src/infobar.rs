//! # InfoBar
//!
//! Module InfoBar provides structures and functions for building and rendering an infobar

use crate::paths::Paths;

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
    /// Updates the infobar
    /// if cmd isn't empty the user is in command mode and therefore that should be displayed
    /// instead of normal mode information. Normal mode information is the index and the current
    /// image path
    pub fn update(paths: &Paths, msg: Option<&str>) -> Self {
        let information: String;
        let mode: String;
        match msg {
            Some(msg) => {
                // user is in command mode
                information = msg.to_string();
                mode = String::from("Command");
            }
            None => {
                // user is in normal mode
                information = match paths.images.get(paths.index) {
                    Some(path) => match path.to_str() {
                        Some(name) => name.to_string(),
                        None => "No file".to_string(),
                    },
                    None => "No file selected".to_string(),
                };
                mode = if paths.images.is_empty() {
                    "No files in path".to_string()
                } else {
                    format!("{} of {}", paths.index + 1, paths.max_viewable)
                };
            }
        }
        Text { information, mode }
    }
}
