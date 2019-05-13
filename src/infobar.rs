//! # InfoBar
//!
//! Module InfoBar provides structures and functions for building and rendering an infobar

use crate::paths::Paths;

/// Text contains the strings required to print the infobar.
pub struct Text {
    /// current image is the string of the current image path
    pub current_image: String,
    /// index is the string represention of the index.
    pub index: String,
}

impl From<&Paths> for Text {
    fn from(p: &Paths) -> Self {
        let current_image = match p.images.get(p.index) {
            Some(path) => match path.to_str() {
                Some(name) => name.to_string(),
                None => "No file".to_string(),
            },
            None => "No file selected".to_string(),
        };
        let current_dir = match p.current_dir.to_str() {
            Some(dir) => dir.to_string(),
            None => "Unknown directory".to_string(),
        };
        let index = if p.images.is_empty() {
            "No files in path".to_string()
        } else {
            format!("{} of {}", p.index + 1, p.images.len())
        };
        Text {
            current_image,
            index,
        }
    }
}
