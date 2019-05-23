//! Paths contains the Paths struct which contains all path related information required for the
//! running of the program.

use shellexpand::full;
use std::path::PathBuf;

/// Paths contains all path information related to the running of the program.
pub struct Paths {
    /// images is the paths of all the images currently in the program.
    pub images: Vec<PathBuf>,
    /// dest_folder is the path of the destination folder for moving and copying images.
    pub dest_folder: PathBuf,
    /// dest_folder was modified from the default keep through Command mode df or destfolder
    pub changed_dest_folder: bool,
    /// current_dir is the path of the current directory where the program was launched from
    pub base_dir: PathBuf,
    /// index is the index of the images vector of the current image to be displayed.
    pub index: usize,
    /// Artificial user facing length of images limited by max cli argument
    pub max_viewable: usize,
    /// Actual length the user said was maximum for images
    pub actual_max_viewable: usize,
}

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
