//! # CLI
//!
//! The cli module is used for setting up the command line app and parsing the arguments.

use clap::{App, Arg};
use glob::glob;
use std::path::PathBuf;

/// cli sets up the command line app and parses the arguments, using clap.
pub fn cli() -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let matches = App::new("clive")
        .version("0.1.0")
        .about("A simple Command Line Image Viewer Executable")
        .arg(
            Arg::with_name("path")
                .required(true)
                .default_value("*")
                .help("The directory or files to search for image files"),
        )
        .get_matches();
    let glob_value = matches
        .value_of("path")
        .expect("Failed to determine a path from the command line arguments");
    let glob_matches = glob(glob_value).expect("Provided path value is not a valid glob");
    for path in glob_matches {
        match path {
            Ok(p) => {
                if let Some(ext) = p.extension() {
                    if ext == "jpg" || ext == "png" || ext == "bmp" {
                        files.push(p)
                    }
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
    Ok(files)
}
