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
use std::path::PathBuf;

/// Converts the provided path by user to a path that can be glob'd
/// Directories are changed from /home/etc to /home/etc/*
pub fn convert_to_globable(path: &str) -> Result<String, String> {
    let expanded_path = full(path).map_err(|e| format!("\"{}\": {}", e.var_name, e.cause))?;
    // remove escaped spaces
    let absolute_path = String::from(expanded_path).replace(r"\ ", " ");
    // If path is a dir, add /* to glob
    let mut pathbuf = PathBuf::from(&absolute_path);
    if pathbuf.is_dir() {
        pathbuf = pathbuf.join("*");
    }
    Ok(pathbuf.to_string_lossy().to_string())
}
