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
    index: Option<usize>,
    /// Artificial user facing length of images limited by max cli argument. None is unlimited
    soft_cap: Option<usize>,
    /// Actual length the user said was maximum for images
    soft_cap_set: Option<usize>,
}

impl PathsBuilder {
    /// Construct an image tracker with no soft cap on images
    pub fn new(images: Vec<PathBuf>, dest_folder: PathBuf, base_dir: PathBuf) -> Self {
        let index = match images.len() {
            0 => None,
            // Default index is first image
            _ => Some(0),
        };
        let soft_cap = Some(images.len());
        let soft_cap_set = Some(images.len());

        Self {
            images,
            dest_folder,
            base_dir,
            index,
            soft_cap,
            soft_cap_set,
        }
    }

    /// Caps maximum images viewable even if the glob found more
    pub fn with_maximum_viewable(mut self, cap: usize) -> Self {
        match cap {
            0 => {
                if self.images.is_empty() {
                    self.soft_cap = None;
                    self.soft_cap_set = None;
                } else {
                    self.soft_cap = Some(self.images.len());
                    self.soft_cap_set = Some(self.images.len());
                }
            }
            _ => {
                self.soft_cap = Some(cap);
                self.soft_cap_set = Some(cap);
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
            soft_cap: self.soft_cap,
            soft_cap_set: self.soft_cap_set,
        }
    }
}

/// Paths contains all path information related to the running of the program.
#[derive(Debug)]
pub struct Paths {
    /// images is the paths of all the images currently in the program.
    images: Vec<PathBuf>,
    /// dest_folder is the path of the destination folder for moving and copying images.
    pub dest_folder: PathBuf,
    /// current_dir is the path of the current directory where the program was launched from
    pub base_dir: PathBuf,
    /// index is the index of the images vector of the current image to be displayed.
    index: Option<usize>,
    /// Artificial user facing length of images limited by max cli argument. 0 is unlimited
    soft_cap: Option<usize>,
    /// Actual length the user said was maximum for images
    soft_cap_set: Option<usize>,
}

impl Paths {
    /// Immutable access to the images globbed
    pub fn images(&self) -> &[PathBuf] {
        self.images.as_slice()
    }

    /// Access image paths mutably
    /// Allows for modification without removal or appending elements
    pub fn images_as_mut_slice(&mut self) -> &mut [PathBuf] {
        self.images.as_mut_slice()
    }

    /// Panics if passed an index bigger than vector holds
    /// Panics if no images are present
    /// Panics if passed an index bigger than max allowed viewable
    pub fn set_index(&mut self, i: usize) {
        if i > self.max_viewable_index().unwrap() {
            panic!(
                "{} is past bounds of images {}",
                i,
                self.max_viewable_index().unwrap()
            );
        }
        self.index = Some(i);
    }

    /// Index of currently viewed image
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    /// Replace the image paths with a new set of them
    pub fn reload_images(&mut self, new_images: Vec<PathBuf>) {
        // Replace the image collection with new ones
        self.images = new_images;

        // Make sure to reset the index
        match self.images.len() {
            0 => self.index = None,
            len => self.index = Some(len - 1),
        }
    }

    /// Reverses images and updates index to keep current image as index
    pub fn reverse(&mut self) {
        self.images.reverse();
        let max_viewable_index = match self.max_viewable_index() {
            Some(i) => i,
            None => return,
        };

        // Safe to unwrap as there is at least one image
        let reversed_index = max_viewable_index - self.index.unwrap();
        self.set_index(reversed_index);
    }

    /// Returns current index and image path
    pub fn current_index_and_path(&self) -> Option<(usize, &PathBuf)> {
        let index = self.index?;
        let path = self.images.get(index)?;
        Some((index, path))
    }

    /// Number of current image (ex. image 1 of 2)
    /// Panics if index is greater than usize
    pub fn current_image(&self) -> Option<usize> {
        Some(self.index?.checked_add(1).unwrap())
    }

    /// Returns the file path for the current image
    /// None if there are no images present
    pub fn current_image_path(&self) -> Option<&PathBuf> {
        let current_index = self.index()?;
        // Will succeed since we have a valid index
        Some(&self.images[current_index])
    }
    /// Maximum viewable index
    /// Returns None if 0 images to display
    pub fn max_viewable_index(&self) -> Option<usize> {
        // Account for if we only have 1 image
        Some(self.max_viewable()?.saturating_sub(1))
    }
    /// Artificial user facing length of images limited by max cli argument
    pub fn max_viewable(&self) -> Option<usize> {
        let true_len = self.images.len();
        if true_len == 0 {
            None
        } else {
            let art_len = self.soft_cap?;
            Some(std::cmp::min(true_len, art_len))
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
            // Set current image index to None if no images left
            // Otherwise decrement the index normally
            self.index = self.index.unwrap().checked_sub(1);
        }
    }

    /// Removes current image from tracking
    pub fn remove_current_image(&mut self) {
        // Check if there are any images to remove
        if let Some(index) = self.index() {
            self.remove_image(index);
        }
    }

    /// Adjusts the index of the current image to view
    /// Does not overflow
    pub fn decrement(&mut self, step: usize) {
        if let Some(index) = self.index() {
            self.index = Some(index.saturating_sub(step));
        }
    }

    /// Increment the currently tracked image
    pub fn increment(&mut self, step: usize) {
        let new_index = match self.index {
            Some(i) => i + step,
            // Can't increment if no images
            None => return,
        };
        // Already checked if there were 0 images above
        let max_viewable_index = self.max_viewable_index().unwrap();

        // Cap new index at soft cap
        if new_index >= max_viewable_index {
            self.index = Some(max_viewable_index);
        } else {
            self.index = Some(new_index);
        }
    }

    /// Update at runtime the maximum images to display at once
    /// Updates index to `actual_max_viewable` if index is too large
    pub fn set_actual_maximum(&mut self, max: usize) {
        if max == 0 {
            self.soft_cap_set = Some(self.images.len());
            self.soft_cap = Some(self.images.len());
            return;
        }
        self.soft_cap_set = Some(max);
        self.soft_cap = Some(max);

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
    fn test_decrement_gives_correct_index() {
        let mut images = dummy_paths_builder(10).build();
        images.decrement(99);
        assert_eq!(images.index(), Some(0));
    }

    #[test]
    fn test_remove_current_image_last_image_no_panic() {
        let mut images = dummy_paths_builder(50).build();
        images.increment(60);
        images.remove_current_image();
        assert_eq!(images.index(), Some(48));
    }

    #[test]
    fn test_remove_current_image_middle_same_index() {
        let mut images = dummy_paths_builder(50).build();
        images.increment(20);
        assert_eq!(images.index(), Some(20));
        images.remove_current_image();
        assert_eq!(images.index(), Some(20));
    }

    #[test]
    fn test_index_gives_correct_index() {
        let mut images = dummy_paths_builder(99).build();
        images.increment(33);
        images.decrement(88);
        images.increment(2);
        assert_eq!(images.index(), Some(2));
    }
    #[test]
    fn test_max_index_gives_valid_current_image() {
        let max_images = 84;
        let mut images = dummy_paths_builder(max_images).build();
        images.increment(max_images);
        let current_image_number = images.current_image();
        assert_eq!(current_image_number, Some(max_images));
    }

    #[test]
    fn test_max_viewable_index_gives_correct_index() {
        let max_images = 10;
        let mut images = dummy_paths_builder(max_images).build();
        images.increment(max_images);
        let current_image_index = images.max_viewable_index();
        assert_eq!(current_image_index, Some(max_images - 1));
    }
    #[test]
    fn test_max_viewable_gives_highest_soft_cap() {
        let max_images = 50;
        let mut images = dummy_paths_builder(max_images).build();
        images.increment(max_images);
        let current_image_number = images.max_viewable();
        assert_eq!(current_image_number, Some(max_images));
    }

    #[test]
    fn test_setting_new_actual_maximum_and_maximum_also_updates_index() {
        let mut images = dummy_paths_builder(50).with_maximum_viewable(50).build();
        images.increment(50);
        images.set_actual_maximum(20);
        assert_eq!(images.index(), Some(19));
    }

    #[test]
    fn test_setting_zero_actual_maximum_removes_soft_max_no_index_disturb() {
        let mut images = dummy_paths_builder(100).with_maximum_viewable(20).build();
        images.increment(50);
        assert_eq!(images.index(), Some(19));
        images.set_actual_maximum(0);
        images.increment(50);
        assert_eq!(images.index(), Some(69));
        // Still capped at last image
        images.increment(999);
        assert_eq!(images.index(), Some(99));
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

        images.increment(1);
        assert_eq!(images.index(), Some(1));
        assert_eq!(images.max_viewable(), Some(50));
        assert_eq!(images.soft_cap_set, Some(50));
    }
}
