//! # RIV
//!
//! The riv library exposes all the functionality required to run the riv command line image viewer
//! app

#![deny(
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_must_use
)]

#[macro_use]
extern crate clap;
extern crate regex;
#[macro_use]
extern crate lazy_static;

pub mod cli;
pub mod infobar;
pub mod paths;
pub mod program;
pub mod screen;
pub mod sort;
pub mod ui;

use regex::Regex;
use shellexpand::full;
use std::path::PathBuf;

/// Converts the provided path by user to a path that can be glob'd, note this function takes the
/// current_directory in order to handle relative paths
/// Paths are normalized removing ".." and "."
/// Environment variables, like ~ and $HOME, are expanded
/// On Unix escaped spaces are removed for example: folder\ path -> folder path
/// Directories are changed from /home/etc to /home/etc/*
/// Symlinks are followed
pub fn path_to_glob(current_dir: &PathBuf, path: &str) -> Result<PathBuf, String> {
    const GLOB: &str = "*";

    let mut expanded_path = match full(path) {
        Ok(path) => {
            let mut path_str = path.to_string();
            // remove escaped characters for Unix
            if cfg!(unix) {
                lazy_static! {
                    static ref REGEX_REMOVE_ESCAPED_CHARS: Regex = match Regex::new(r"\\(.)") {
                        Ok(regex) => regex,
                        Err(e) => panic!("Logic Error: {}", e),
                    };
                }
                path_str = REGEX_REMOVE_ESCAPED_CHARS
                    .replace_all(&path_str, "$1")
                    .to_string();
            }
            PathBuf::from(&path_str)
        }
        Err(e) => return Err(format!("\"{}\": {}", e.var_name, e.cause)),
    };

    if expanded_path.is_relative() {
        expanded_path = current_dir.join(expanded_path);
    }
    let mut expanded_path = normalize_path(expanded_path);
    // If path is a dir, add /* to glob
    if expanded_path.is_dir() {
        expanded_path.push(GLOB);
    }
    Ok(expanded_path)
}

/// Normalizes paths removing "." and ".."
/// This follows symlinks like std::fs::canonicalize
/// This is a helper function to path_to_glob
///
///
/// This implementation is preferred to using std::fs::canonicalize due to canonicalize requiring
/// the path to exist, causing all globbing to fail and having to remove the glob, normalize then add it back on
fn normalize_path(path: PathBuf) -> PathBuf {
    use std::fs::read_link;
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => continue,
            _ => {
                normalized.push(component);
                if let Ok(actual_path) = read_link(&normalized) {
                    normalized.set_file_name(actual_path);
                }
            }
        }
    }
    normalized
}

/// Takes in the output of path_to_glob and finds the closest parent in that path
/// This is the new base directory
pub fn new_base_dir(path: &PathBuf) -> Result<PathBuf, String> {
    for parent in path.ancestors() {
        if parent.is_dir() {
            return Ok(parent.to_path_buf());
        }
    }
    Err(format!(
        "Failed to get new base directory for path: \"{}\"",
        path.display()
    ))
}
