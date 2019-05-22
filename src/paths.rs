//! Paths contains the Paths struct which contains all path related information required for the
//! running of the program.

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
