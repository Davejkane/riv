//! # CLI
//!
//! The cli module is used for setting up the command line app and parsing the arguments.

// Only because of SortOrder, as a result of Clap bug/restriction that only permits bare variants
// inside of arg!_enum macro call
#![allow(missing_docs)]

use clap::{App, Arg};
use std::path::PathBuf;

arg_enum! {
    /// Enum used by clap cli app, in order to parse sorting options
    pub enum SortOrder {
        // Alphabetically by filename only
        Alphabetical,
        // Current directory images first, followed by subdirectories
        BreadthFirst,
        // By Modified date, most recent first
        Date,
        // [Default] Farthest depth images first
        DepthFirst,
        // By Size, largest size first
        Size,
    }
}

/// Args contains the arguments that have been successfully parsed by the clap cli app
pub struct Args {
    /// files is the vector of image file paths that match the supplied or default glob
    pub files: Vec<PathBuf>,
    /// dest_folder is the supplied or default folder for moving files
    pub dest_folder: PathBuf,
    /// Provides the SortOrder specified by the user
    pub sort_order: SortOrder,
    /// whether or not to reverse sorting
    pub reverse: bool,
}

/// cli sets up the command line app and parses the arguments, using clap.
pub fn cli() -> Result<Args, String> {
    let mut files = Vec::new();
    let matches = App::new("riv")
        .version("0.2.0")
        .about("The command line image viewer")
        .arg(
            Arg::with_name("paths")
                .multiple(true)
                .help("The directory or files to search for image files. A glob can be used here."),
        )
        .arg(
            Arg::with_name("dest-folder")
                .default_value("./keep")
                .short("f")
                .long("dest-folder")
                .help("Desintation folder for moving files to")
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
        .get_matches();
    match matches.values_of("paths") {
        Some(path_matches) => {
            for path in path_matches {
                push_image_path(&mut files, PathBuf::from(path));
            }
        }
        None => {
            let path_matches = glob::glob("*").map_err(|e| e.to_string())?;
            for path in path_matches {
                match path {
                    Ok(p) => {
                        push_image_path(&mut files, p);
                    }
                    Err(e) => eprintln!("Unexpected path {}", e),
                }
            }
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
        None => return Err("failed to determine destintation folder".to_string()),
    };

    let reverse = matches.is_present("reverse");

    Ok(Args {
        files,
        dest_folder,
        sort_order,
        reverse,
    })
}

fn push_image_path(v: &mut Vec<PathBuf>, p: PathBuf) {
    if let Some(ext) = p.extension() {
        if let Some(ext_str) = ext.to_str() {
            let low = ext_str.to_string().to_lowercase();
            if low == "jpg" || low == "jpeg" || low == "png" || low == "bmp" || low == "webp" {
                v.push(p)
            }
        }
    }
}
