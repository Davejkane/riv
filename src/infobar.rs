//! # InfoBar
//!
//! Module InfoBar provides structures and functions for building and rendering an infobar

use crate::paths::Paths;

/// Text contains the strings required to print the infobar.
pub struct Text {
    /// current image is the string of the current image path
    pub current_image: String,
    /// current_dir is the string of the current directory from where the program was launched
    pub current_dir: String,
    /// glob is the string of the glob that was used for path search
    pub glob: String,
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
            None => "No file".to_string(),
        };
        let current_dir = match p.current_dir.to_str() {
            Some(dir) => dir.to_string(),
            None => "Unknown directory".to_string(),
        };
        let glob = p.glob.clone();
        let index = format!("{} of {}", p.index + 1, p.images.len());
        Text {
            current_image,
            current_dir,
            glob,
            index,
        }
    }
}
