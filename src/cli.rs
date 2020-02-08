//! # CLI
//!
//! The cli module is used for setting up the command line app and parsing the arguments.

use crate::sort::SortOrder;
use clap::{App, Arg};
use glob::glob;
use std::env::current_dir;
use std::path::PathBuf;

/// Args contains the arguments that have been successfully parsed by the clap cli app
pub struct Args {
    /// files is the vector of image file paths that match the supplied or default glob
    pub files: Vec<PathBuf>,
    /// dest_folder is the supplied or default folder for moving files
    pub dest_folder: PathBuf,
    /// provides the SortOrder specified by the user
    pub sort_order: SortOrder,
    /// whether or not to reverse sorting
    pub reverse: bool,
    /// maximum length of files to display
    pub max_length: usize,
    /// Start in fullscreen mode
    pub fullscreen: bool,
    /// New base directory defaults to std::env::current_dir
    pub base_dir: PathBuf,
}

/// cli sets up the command line app and parses the arguments, using clap.
pub fn cli() -> Result<Args, String> {
    let mut files = Vec::new();
    let matches = App::new("riv")
        .version("0.3.0")
        .about("The command line image viewer")
        .arg(
            Arg::with_name("paths")
                .default_value("*")
                .help("The directory or files to search for image files. A glob can be used here."),
        )
        .arg(
            Arg::with_name("dest-folder")
                .default_value("./keep")
                .short("f")
                .long("dest-folder")
                .help("Destination folder for moving files to")
                .takes_value(true),
        )
        .arg(
            Arg::from_usage("<sort-order> 'Sorting order to use'")
                .default_value("DepthFirst")
                .short("s")
                .long("sort")
                .takes_value(true)
                .case_insensitive(true)
                .possible_values(&SortOrder::variants())
                .help("Sort order for images"),
        )
        .arg(
            Arg::with_name("reverse")
                .default_value("false")
                .short("r")
                .long("reverse")
                .help("Reverses the sorting of images")
                .multiple(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("max-number-images")
                .default_value("0")
                .short("m")
                .long("max")
                .help("The maximum numbers of images to display [0 means infinitely many]")
                .multiple(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fullscreen")
                .takes_value(false)
                .long("fullscreen")
                .short("F")
                .help("Start app in fullscreen mode"),
        )
        .get_matches();

    let path_glob = match matches.value_of("paths") {
        Some(v) => v,
        None => panic!("No value for paths!"),
    };
    // find current directory so glob provided can be relative
    let mut base_dir = match current_dir() {
        Ok(c) => c,
        Err(_) => PathBuf::new(),
    };
    let path_glob = crate::path_to_glob(&base_dir, path_glob)?;
    // find new base directory
    if let Ok(new_base_dir) = crate::new_base_dir(&path_glob) {
        base_dir = new_base_dir;
    }
    let glob_matches = glob(&path_glob.to_string_lossy()).map_err(|e| e.to_string())?;
    for path in glob_matches {
        match path {
            Ok(p) => push_image_path(&mut files, p),
            Err(e) => eprintln!("Path not processable {}", e),
        }
    }

    let sort_order = match value_t!(matches, "sort-order", SortOrder) {
        Ok(order) => order,
        Err(e) => {
            eprintln!("{}", e);
            SortOrder::DepthFirst
        }
    };

    let dest_folder = match matches.value_of("dest-folder") {
        Some(f) => PathBuf::from(f),
        None => return Err("failed to determine destination folder".to_string()),
    };

    let reverse = matches.is_present("reverse");

    let max_length = value_t!(matches, "max-number-images", usize).unwrap_or(0);
    let fullscreen = matches.is_present("fullscreen");

    Ok(Args {
        files,
        dest_folder,
        sort_order,
        reverse,
        max_length,
        fullscreen,
        base_dir,
    })
}

pub(crate) fn push_image_path(v: &mut Vec<PathBuf>, p: PathBuf) {
    if let Some(ext) = p.extension() {
        if let Some(ext_str) = ext.to_str() {
            let low = ext_str.to_string().to_lowercase();
            match low.as_ref() {
                "jpg" | "jpeg" | "png" | "bmp" | "webp" | "gif" | "lbm" | "pcx" | "pnm" | "svg"
                | "tga" | "tiff" | "xcf" | "xpm" | "xv" => v.push(p),
                _ => {
                    eprintln!("{} is not a supported image format", low);
                }
            }
        }
    }
}
