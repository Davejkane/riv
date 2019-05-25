//! Paths contains the Paths struct which contains all path related information required for the
//! running of the program.

use std::path::PathBuf;

/// Builds a new Paths
#[derive(Debug)]
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
    max_viewable: Option<usize>,
    /// Actual length the user said was maximum for images
    actual_max_viewable: Option<usize>,
}
/// Paths contains all path information related to the running of the program.
#[derive(Debug)]
pub struct Paths {
    /// images is the paths of all the images currently in the program.
    pub images: Vec<PathBuf>,
    /// dest_folder is the path of the destination folder for moving and copying images.
    pub dest_folder: PathBuf,
    /// current_dir is the path of the current directory where the program was launched from
    pub base_dir: PathBuf,
    /// index is the index of the images vector of the current image to be displayed.
    index: usize,
    /// Artificial user facing length of images limited by max cli argument. 0 is unlimited
    max_viewable: Option<usize>,
    /// Actual length the user said was maximum for images
    actual_max_viewable: Option<usize>,
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
    pub fn with_maximum_viewable(mut self, cap: usize) -> Self {
        match cap {
            0 => {
                self.max_viewable = None;
                self.actual_max_viewable = None;
            }
            _ => {
                self.max_viewable = Some(cap);
                self.actual_max_viewable = Some(cap);
            }
        };
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
    /// Panics if passed an index bigger than vector holds
    /// Panics if passed an index bigger than max allowed viewable
    pub fn set_index(&mut self, i: usize) {
        if i > self.max_viewable_index() {
            panic!(
                "{} is past bounds of images {}",
                i,
                self.max_viewable_index()
            );
        }
        self.index = i;
    }

    /// Index of currently viewed image
    pub fn index(&self) -> usize {
        self.index
    }

    /// Number of current image (ex. image 1 of 2)
    /// Panics if index is greater than usize
    pub fn current_image(&self) -> usize {
        self.index.checked_add(1).unwrap()
    }
    /// Maximum viewable index
    /// Returns 0 if 0 images to display
    pub fn max_viewable_index(&self) -> usize {
        // Account for if we only have 1 image
        self.max_viewable().saturating_sub(1)
    }
    /// Artificial user facing length of images limited by max cli argument
    pub fn max_viewable(&self) -> usize {
        let true_len = self.images.len();
        match self.max_viewable {
            Some(art_cap) => std::cmp::min(true_len, art_cap),
            None => true_len,
        }
    }

    /// Removes an image from tracked images.
    /// Upholds that the index should always be <= index of last image.
    ///
    /// # Panics
    ///
    /// Panics if `index` tries to access past `self.images` bounds
    pub fn remove_image(&mut self, index: usize) {
        let len = self.images.len();
        assert!(index < len);

        // Remove image
        self.images.remove(index);

        let new_len = self.images.len();
        // Adjust index if past bounds
        if index >= new_len {
            // Prevent underflow by flooring at 0 index
            self.index = self.index.saturating_sub(1);
        }
    }

    /// Removes current image from tracking
    pub fn remove_current_image(&mut self) {
        self.remove_image(self.index);
    }

    /// Adjusts the index of the current image to view
    /// Does not overflow
    pub fn decrement(&mut self, step: usize) {
        self.index = self.index.saturating_sub(step);
    }

    /// Increment the currently tracked image
    pub fn increment(&mut self, step: usize) {
        let new_index = self.index + step;

        // Cap new index at soft cap
        if new_index >= self.max_viewable_index() {
            self.index = self.max_viewable_index();
        } else {
            self.index = new_index;
        }
    }

    /// Update at runtime the maximum images to display at once
    /// Updates index to `actual_max_viewable` if index is too large
    pub fn set_actual_maximum(&mut self, max: usize) {
        if max == 0 {
            self.actual_max_viewable = None;
            self.max_viewable = None;
            return;
        }
        self.actual_max_viewable = Some(max);
        self.max_viewable = Some(max);

        // Cap index if new max is smaller
        if self.index > self.max_viewable_index() {
            self.index = self.max_viewable_index();
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
    fn test_remove_current_image_last_image_no_panic() {
        let mut images = dummy_paths_builder(50).build();
        images.increment(60);
        images.remove_current_image();
        assert_eq!(images.index(), 48);
    }

    #[test]
    fn test_remove_current_image_middle_same_index() {
        let mut images = dummy_paths_builder(50).build();
        images.increment(20);
        assert_eq!(images.index(), 20);
        images.remove_current_image();
        assert_eq!(images.index(), 20);
    }

    #[test]
    fn test_index_gives_correct_index() {
        let mut images = dummy_paths_builder(99).build();
        images.increment(33);
        images.decrement(88);
        images.increment(2);
        assert_eq!(images.index(), 2);
    }
    #[test]
    fn test_max_index_gives_valid_current_image() {
        let max_images = 84;
        let mut images = dummy_paths_builder(max_images).build();
        images.increment(max_images);
        let current_image_number = images.current_image();
        assert_eq!(current_image_number, max_images);
    }

    #[test]
    fn test_setting_new_actual_maximum_and_maximum_also_updates_index() {
        let mut images = dummy_paths_builder(50).with_maximum_viewable(50).build();
        images.increment(50);
        images.set_actual_maximum(20);
        assert_eq!(images.index(), 19);
    }

    #[test]
    fn test_setting_zero_actual_maximum_removes_soft_max_no_index_disturb() {
        let mut images = dummy_paths_builder(100).with_maximum_viewable(20).build();
        images.increment(50);
        assert_eq!(images.index(), 19);
        images.set_actual_maximum(0);
        images.increment(50);
        assert_eq!(images.index(), 69);
        // Still capped at last image
        images.increment(999);
        assert_eq!(images.index(), 99);
    }

    #[test]
    fn test_soft_maximum_view() {
        let mut images = dummy_paths_builder(50).with_maximum_viewable(10).build();
        images.increment(20);
        assert_eq!(images.index, images.max_viewable_index());
    }

    #[test]
    fn test_increment_image() {
        let mut images = dummy_paths_builder(50).build();
        const STARTING_INDEX: usize = 0;
        const STEP: usize = 1;

        images.increment(1);
        assert_eq!(images.index, STARTING_INDEX + STEP);
        assert_eq!(images.max_viewable(), 50);
        assert_eq!(images.actual_max_viewable, None);
    }
}
