//! Paths contains the Paths struct which contains all path related information required for the
//! running of the program.

use std::path::PathBuf;

/// Builds a new Paths
pub struct PathsBuilder {
    /// images is the paths of all the images currently in the program.
    images: Vec<PathBuf>,
    /// dest_folder is the path of the destination folder for moving and copying images.
    dest_folder: PathBuf,
    /// current_dir is the path of the current directory where the program was launched from
    base_dir: PathBuf,
    /// index is the index of the images vector of the current image to be displayed.
    index: usize,
    /// Artificial user facing length of images limited by max cli argument. 0 is unlimited
    max_viewable: usize,
    /// Actual length the user said was maximum for images
    actual_max_viewable: usize,
}
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
    /// Artificial user facing length of images limited by max cli argument. 0 is unlimited
    max_viewable: Some(usize),
    /// Actual length the user said was maximum for images
    actual_max_viewable: Some(usize),
}

impl Paths {
    fn index(&self) -> usize {
        self.index
    }
}

impl PathsBuilder {
    /// Construct an image tracker with no soft cap on images
    pub fn new(images: Vec<PathBuf>, dest_folder: PathBuf, base_dir: PathBuf) -> Self {
        let index = 0;
        let max_viewable = None;
        let actual_max_viewable = None;

        Self {
            images,
            dest_folder,
            base_dir,
            index,
            max_viewable,
            actual_max_viewable,
        }
    }

    /// Caps maximum images viewable even if the glob found more
    pub fn with_soft_cap(mut self, cap: usize) -> Self {
        self.soft_cap = cap;
        self
    }

    /// Build the Paths struct
    pub fn build(self) -> Paths {
        Paths {
            images: self.images,
            dest_folder: self.dest_folder,
            base_dir: self.base_dir,
            index: self.index,
            max_viewable: self.max_viewable,
            actual_max_viewable: self.actual_max_viewable,
        }
    }
}

impl Paths {
    /// Artificial user facing length of images limited by max cli argument
    pub fn max_viewable(&self) -> usize {
        let true_len = self.images.len();
        if let Some(art_cap) = self.max_viewable {
            if true_len > art_cap {
                art_cap
            }
        }
        true_len
    }

    /// Removes an image from tracked images.
    /// Upholds that the index should always be <= index of last image.
    ///
    /// # Panics
    ///
    /// Panics if `index` tries to access past `self.images` bounds
    fn remove_image(&mut self, index: usize) {
        let len = self.images.len();
        assert!(index < len);

        // Remove image
        self.images.remove(index);

        let new_len = self.images.len();
        // Adjust index if past bounds
        if index >= new_len {
            // Prevent underflow by flooring at 0 index
            self.paths.index.saturating_sub(1);
        }
        // Decrease max viewable if another image won't take the deleted's place
        if self.images.len() <= self.max_viewable {
            self.max_viewable -= 1;
        }
    }

    /// Adjusts the index of the current image to view
    /// Does not overflow
    pub fn decrement(&mut self, step: usize) {
        self.index = self.index.saturating_sub(step);
    }

    /// Increment the currently tracked image
    pub fn increment(&mut self, step: usize) {
        let new_index = self.index + step;
        if new_index >= self.images.len() - 1 {
            self.index = self.max_viewable.saturating_sub(1);
            return;
        }
        // new index would be past soft cap
        if new_index >= self.max_viewable {
            // cap at soft cap
            self.index = self.max_viewable;
        } else {
            self.index = new_index;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::paths::PathsBuilder;
    use std::iter::repeat;
    use std::path::PathBuf;

    fn dummy_paths_builder(n: usize) -> PathsBuilder {
        let images = repeat(PathBuf::new()).take(n).collect::<Vec<PathBuf>>();
        PathsBuilder::new(images, "./keep".into(), ".".into())
    }

    #[test]
    fn test_soft_cap() {
        let mut images = dummy_paths_builder(50).with_soft_cap(10).build();
        images.increment(20);
        assert_eq!(images.index, images.max_viewable);
    }

    #[test]
    fn test_increment_image() {
        let mut images = dummy_paths_builder(50).build();
        const STARTING_INDEX: usize = 0;
        const STEP: usize = 1;

        images.increment(1);
        assert_eq!(images.index, STARTING_INDEX + STEP);
        assert_eq!(images.max_viewable, 50);
        assert_eq!(images.actual_max_viewable, 50);
    }
}
