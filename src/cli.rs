//! # CLI
//!
//! The cli module is used for setting up the command line app and parsing the arguments.

use clap::{App, Arg};
use glob::glob;
use std::path::PathBuf;

/// Args contains the arguments that have been successfully parsed by the clap cli app
pub struct Args {
    /// files is the vector of image file paths that match the supplied or default glob
    pub files: Vec<PathBuf>,
    /// dest_folder is the supplied or default folder for moving files
    pub dest_folder: PathBuf,
    /// glob is the glob used in the image search
    pub search: String,
}

/// cli sets up the command line app and parses the arguments, using clap.
pub fn cli() -> Result<Args, String> {
    let mut files = Vec::new();
    let matches = App::new("riv")
        .version("0.2.0")
        .about("The command line image viewer")
        .arg(
            Arg::with_name("path")
                .required(true)
                .default_value("*")
                .help("The directory or files to search for image files"),
        )
        .arg(
            Arg::with_name("dest-folder")
                .default_value("./keep")
                .short("f")
                .long("dest-folder")
                .help("Desintation folder for moving files to")
                .takes_value(true),
        )
        .get_matches();
    let glob_value = match matches.value_of("path") {
        Some(v) => v,
        None => return Err("Failed to determine glob value".to_string()),
    };
    let glob_matches = glob(glob_value).map_err(|e| e.to_string())?;
    for path in glob_matches {
        match path {
            Ok(p) => {
                if let Some(ext) = p.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        let low = ext_str.to_string().to_lowercase();
                        if low == "jpg"
                            || low == "jpeg"
                            || low == "png"
                            || low == "bmp"
                            || low == "webp"
                        {
                            files.push(p)
                        }
                    }
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
    let dest_folder = match matches.value_of("dest-folder") {
        Some(f) => PathBuf::from(f),
        None => return Err("failed to determine destintation folder".to_string()),
    };
    let search = glob_value.to_owned();
    Ok(Args {
        files,
        dest_folder,
        search,
    })
}
