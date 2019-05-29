//! # Sort
//!
//! Contains Sorter and SortOrder that use command line arguments to sort images

// Only because of SortOrder, as a result of Clap bug/restriction that only permits bare variants
// inside of arg!_enum macro call
#![allow(missing_docs)]

use core::cmp::Ordering;
use fs_extra::dir::get_size;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::SystemTime;

/// A structure that takes the SortOrder from command line arguments
/// and reverse and sorts pathes based on original criteria
// Chosen over functions, due to it being more adaptable in the future
pub struct Sorter {
    /// Order to sort the files
    sort_order: SortOrder,
    /// Whether or not to reverse the order of sorting
    reverse: bool,
}

impl Sorter {
    /// Create sorter with command line arguments SortOrder and reverse
    pub fn new(sort_order: SortOrder, reverse: bool) -> Self {
        Self {
            sort_order,
            reverse,
        }
    }

    /// Change the order of sorting, this function doesn't sort upon change
    pub fn set_order(&mut self, sort_order: SortOrder) {
        self.sort_order = sort_order;
    }

    /// Change whether or not to reverse the sorting of images
    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }

    /// Sorts the images based on sort_order, reverses if necessary
    pub fn sort<'a>(&self, paths: &'a mut [PathBuf]) -> &'a mut [PathBuf] {
        paths.sort_by(|a, b| self.sort_order.file_compare(&a, &b));
        if self.reverse {
            paths.reverse();
        }
        paths
    }
}

// arg_enum! prevents documentation from being written for enum variants
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

impl SortOrder {
    /// A comparator for each of the sorting types returns an ordering
    fn file_compare(&self, a: &PathBuf, b: &PathBuf) -> Ordering {
        match self {
            // if get_size fails just assume that it's size 0
            SortOrder::Size => get_size(b).unwrap_or(0).cmp(&get_size(a).unwrap_or(0)),
            SortOrder::Date => file_get_date(b).cmp(&file_get_date(a)),
            SortOrder::Alphabetical => {
                let a_no_dot = trim_hidden(a.file_stem());
                let b_no_dot = trim_hidden(b.file_stem());
                natord::compare_ignore_case(&a_no_dot, &b_no_dot)
            }
            SortOrder::DepthFirst => calculate_depth(&b).cmp(&calculate_depth(&a)),
            SortOrder::BreadthFirst => calculate_depth(&a).cmp(&calculate_depth(&b)),
        }
    }
}

/// Get the depth of a path + 1 due to the filename being included
fn calculate_depth(path: &PathBuf) -> usize {
    path.ancestors().count()
}

/// if the file is a hidden file or dotfile, remove the dot
fn trim_hidden(file_name: Option<&OsStr>) -> String {
    let name = file_name.unwrap_or_else(|| panic!("User passed a file that doesn't have a name"));
    let lossy_name = name.to_string_lossy();
    if lossy_name.starts_with('.') {
        lossy_name[1..].into()
    } else {
        lossy_name.into()
    }
}

/// Get the time a file was last modified, if this function fails it returns Now
fn file_get_date(path: &PathBuf) -> SystemTime {
    let metadata = match path.metadata() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", e);
            return SystemTime::now();
        }
    };
    match metadata.modified() {
        Ok(time) => time,
        Err(e) => {
            eprintln!("{}", e);
            SystemTime::now()
        }
    }
}
