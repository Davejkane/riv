//! Paths contains the Paths struct which contains all path related information required for the
//! running of the program. As well as utility functions used to add paths and normalize paths

use std::path::PathBuf;

/// Paths contains all path information related to the running of the program.
pub struct Paths {
    /// images is the paths of all the images currently in the program.
    pub images: Vec<PathBuf>,
    /// dest_folder is the path of the destination folder for moving and copying images.
    pub dest_folder: PathBuf,
    /// current_dir is the path of the current directory where the program was launched from
    pub base_dir: PathBuf,
    /// index is the index of the images vector of the current image to be displayed.
    pub index: usize,
    /// Artificial user facing length of images limited by max cli argument
    pub max_viewable: usize,
    /// Actual length the user said was maximum for images
    pub actual_max_viewable: usize,
}

/// Pushes images to vector (v) if the path (p) extension is a valid image extension
pub fn push_image_path(v: &mut Vec<PathBuf>, p: PathBuf) {
    if let Some(ext) = p.extension() {
        if let Some(ext_str) = ext.to_str() {
            let low = ext_str.to_string().to_lowercase();
            if low == "jpg" || low == "jpeg" || low == "png" || low == "bmp" || low == "webp" {
                v.push(p)
            }
        }
    }
}

/// Constructs a normalized_path removing '.', parent_dir/'..'
pub fn normalize_path(path: &PathBuf) -> PathBuf {
    use std::path::Component;

    let mut normalized_path = PathBuf::new();
    for component in path.components() {
        // skip '.'
        if component == Component::CurDir {
            continue;
        // remove last appended file '..'
        } else if component == Component::ParentDir {
            normalized_path.pop();
        } else {
            normalized_path.push(component);
        }
    }
    normalized_path
}
