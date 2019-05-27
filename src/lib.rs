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

pub mod cli;
pub mod infobar;
pub mod paths;
pub mod program;
pub mod screen;
pub mod sort;
pub mod ui;

use shellexpand::full;
use std::fs::canonicalize;
use std::path::PathBuf;

/// Converts the provided path by user to a path that can be glob'd, note this function takes the
/// current_directory in order to handle relative paths
/// Paths are normalized removing ".." and "."
/// Environment variables, like ~ and $HOME, are expanded
/// On Unix escaped spaces are removed for example: folder\ path -> folder path
/// Directories are changed from /home/etc to /home/etc/*
pub fn path_to_glob(current_dir: &PathBuf, path: &str) -> Result<PathBuf, String> {
    const ESCAPED_SPACE: &str = r"\ ";
    const SPACE: &str = " ";

    let mut expanded_path = match full(path) {
        Ok(path) => {
            let mut path_str = path.to_string();
            // remove escaped spaces for Unix
            if cfg!(unix) {
                path_str = path_str.replace(ESCAPED_SPACE, SPACE);
            }
            PathBuf::from(&path_str)
        }
        Err(e) => return Err(format!("\"{}\": {}", e.var_name, e.cause)),
    };

    if expanded_path.is_relative() {
        expanded_path = current_dir.join(expanded_path);
    }
    // normalize path
    let mut expanded_path = match normalize_path(expanded_path) {
        Ok(path) => path,
        Err(e) => return Err(e.to_string()),
    };
    // If path is a dir, add /* to glob
    if expanded_path.is_dir() {
        expanded_path.push("*");
    }
    Ok(expanded_path)
}

/// Normalizes paths removing "." and ".."
/// This is a helper function to path_to_glob
fn normalize_path(path: PathBuf) -> Result<PathBuf, String> {
    if let Ok(path) = canonicalize(&path) {
        Ok(path)
    } else {
        // if canonicalize failed it's most likely because the path contains '*'s
        // remove those and see if it is successful, and add them back on if it is
        use std::ffi::OsStr;

        let mut stack: Vec<&OsStr> = Vec::new();
        let mut invalid_expanded_path = path.clone();
        for parent in path.ancestors() {
            if let Some(child) = parent.file_name() {
                if invalid_expanded_path.exists() {
                    break;
                }
                stack.push(child);
            } else {
                // parent is '..' remove it
                invalid_expanded_path.pop();
            }
            invalid_expanded_path.pop();
        }
        let mut new_path = canonicalize(&invalid_expanded_path).map_err(|e| e.to_string())?;
        while let Some(sub_path) = stack.pop() {
            new_path.push(sub_path);
        }
        Ok(new_path)
    }
}

/// Takes in the output of path_to_glob and finds the closest parent in that path
/// This is the new base directory
pub fn new_base_dir(path: &PathBuf) -> Result<PathBuf, String> {
    for parent in path.ancestors() {
        if parent.is_dir() {
            return Ok(parent.to_path_buf());
        }
    }
    Err(format!("Failed to get new base directory"))
}
