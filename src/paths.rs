//! Paths contains the Paths struct which contains all path related information required for the
//! running of the program.

use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::slice::SliceIndex;

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
    /// Artificial user facing length of images limited by max cli argument
    art_len: usize,
    /// Original Artificial length of the vector
    art_len_orig: Option<usize>,
}

impl PathsBuilder {
    /// Construct an image tracker with no soft cap on images
    pub fn new(images: Vec<PathBuf>, dest_folder: PathBuf, base_dir: PathBuf) -> Self {
        let index = match images.len() {
            0 => None,
            // Default index is first image
            _ => Some(0),
        };
        let art_len = images.len();
        let art_len_orig = None;

        Self {
            images,
            dest_folder,
            base_dir,
            index,
            art_len,
            art_len_orig,
        }
    }

    /// Caps maximum images viewable even if the glob found more
    pub fn with_maximum_viewable(mut self, cap: usize) -> Self {
        match cap {
            0 => {
                self.art_len = self.images.len();
                self.art_len_orig = None;
            }
            _ => {
                self.art_len = std::cmp::min(cap, self.images.len());
                self.art_len_orig = Some(cap);
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
            art_len: self.art_len,
            art_len_orig: self.art_len_orig,
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
    /// Artificial user facing length of images limited by max cli argument
    art_len: usize,
    /// Original Artificial length of the vector
    art_len_orig: Option<usize>,
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

    /// Sets which image index is the current image
    ///
    /// # Panics
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

    /// Get the image path at the specified index range
    /// If the range is greater than the maximum viewable index
    /// it is capped at the maximum viewable index
    /// None is returned if there are no images
    pub fn get_range(&self, range: &RangeInclusive<usize>) -> Option<&[PathBuf]> {
        let max_index = self.max_viewable_index()?;
        if *range.start() > max_index {
            return None;
        }

        let new_start = *range.start();
        let new_end = if *range.end() > max_index {
            // cap at max index
            max_index
        } else {
            *range.end()
        };

        Some(self.get(new_start..=new_end).unwrap())
    }

    // Copied signature from stdlib Vec get
    // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.get
    /// Immutable access to any image in images storage. Bypasses max viewable check
    fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[PathBuf]>,
    {
        self.images.get(index)
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
            0 => {
                self.index = None;
                self.art_len = 0;
            }
            _ => {
                self.index = Some(0);
                self.art_len = match self.art_len_orig {
                    Some(orig_art_len) => std::cmp::min(orig_art_len, self.images.len()),
                    None => self.images.len(),
                };
            }
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
    /// The image number of the last image viewable
    /// Returns None if no image are present
    pub fn max_viewable(&self) -> Option<usize> {
        if self.art_len == 0 {
            None
        } else {
            Some(self.art_len)
        }
    }

    /// Removes an image from tracked images.
    /// Upholds that the index should always be <= index of last image.
    ///
    /// # Panics
    ///
    /// Panics if `index` tries to access past `self.images` bounds
    pub fn remove_image(&mut self, index: usize) {
        let len =  match self.max_viewable() {
            Some(len) => len,
            // No images to remove
            None => return,
        };

        assert!(index < len);
        // Remove image
        self.images.remove(index);
        // Decrease artificial length
        self.art_len = self.art_len.saturating_sub(1);

        let new_len = len - 1;
        // Set index to None if no images left
        if new_len == 0 {
            self.index = None;
            return;
        }
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

    /// Decrement which image is currently tracked
    /// Floors at the first image (0 index)
    /// Does nothing if no images are present
    pub fn decrement(&mut self, step: usize) {
        if let Some(index) = self.index() {
            self.index = Some(index.saturating_sub(step));
        }
    }

    /// Advance which image is currently tracked
    pub fn increment(&mut self, step: usize) {
        let new_index = match self.index {
            Some(i) => i + step,
            // Can't increment if no images
            None => return,
        };
        // Already checked if there were 0 images above
        let max_viewable_index = self.max_viewable_index().unwrap();

        // Cap new index at artificial length
        if new_index >= max_viewable_index {
            self.index = Some(max_viewable_index);
        } else {
            self.index = Some(new_index);
        }
    }

    /// Update at runtime the maximum images to display at once
    /// Updates index to `actual_max_viewable` if index is too large
    pub fn set_actual_maximum(&mut self, art_max: usize) {
        if art_max == 0 {
            self.art_len_orig = None;
            self.art_len = self.images.len();
            return;
        }
        self.art_len_orig = Some(art_max);
        self.art_len = std::cmp::min(art_max, self.images.len());

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
    fn test_no_artificial_cap_reload_with_no_existing_and_new_images() {
        let mut images = dummy_paths_builder(0).build();
        let less_images = dummy_paths_builder(0).build();
        images.reload_images(less_images.images);
        assert_eq!(images.art_len, 0);
        assert_eq!(images.index, None);
        assert_eq!(images.max_viewable(), None);
    }

    #[test]
    fn test_no_artificial_cap_reload_with_no_existing_and_some_new_images() {
        let mut images = dummy_paths_builder(0).build();
        let more_images = dummy_paths_builder(100).build();
        images.reload_images(more_images.images);
        assert_eq!(images.art_len, 100);
        assert_eq!(images.index, Some(0));
        assert_eq!(images.max_viewable(), Some(100));
    }
    #[test]
    fn test_removing_all_images_with_art_len_and_spare_has_no_index_or_max_viewable() {
        let mut images = dummy_paths_builder(10).with_maximum_viewable(2).build();
        images.increment(3);
        images.remove_current_image();
        images.remove_current_image();
        assert_eq!(images.current_image(), None);
        assert_eq!(images.max_viewable(), None);
    }

    #[test]
    fn test_current_image_number_when_remove_last_image_with_one_left_with_artificial_len() {
        let mut images = dummy_paths_builder(10).with_maximum_viewable(2).build();
        images.increment(3);
        images.remove_current_image();
        assert_eq!(images.current_image(), Some(1));
    }

    #[test]
    fn test_get_indexes() {
        let images = dummy_paths_builder(10).build();
        let slice = images.get(5);
        let multi_slice = images.get(1..=5);
        dbg!(slice);
        dbg!(multi_slice);
        assert_eq!(slice, Some(&PathBuf::from("")));
        assert_eq!(multi_slice.unwrap().len(), 5);
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
    fn test_max_viewable_with_lower_artificial_len_gives_lower_len() {
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
    fn test_setting_zero_actual_maximum_removes_sets_art_len_to_all_images_no_index_disturb() {
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
    fn test_artificial_len_maximum_view() {
        let mut images = dummy_paths_builder(50).with_maximum_viewable(10).build();
        images.increment(20);
        assert_eq!(images.index, images.max_viewable_index());
    }

    #[test]
    fn test_artificial_maximum_limits_current_glob_on_image_delete() {
        let mut images = dummy_paths_builder(50).with_maximum_viewable(2).build();
        images.increment(99);
        images.remove_current_image();
        assert_eq!(images.max_viewable_index(), Some(0));
    }

    #[test]
    fn test_artificial_len_after_delete_at_end() {
        let mut images = dummy_paths_builder(50).build();
        images.increment(99);
        images.remove_current_image();
        assert_eq!(images.max_viewable_index(), Some(48));
    }

    #[test]
    fn test_increment_image() {
        let mut images = dummy_paths_builder(50).build();

        images.increment(1);
        assert_eq!(images.index(), Some(1));
        assert_eq!(images.max_viewable(), Some(50));
        assert_eq!(images.art_len_orig, None);
    }

    #[test]
    fn test_replacing_images_resets_index() {
        let mut images = dummy_paths_builder(50).build();
        let less_images = dummy_paths_builder(10).build();

        images.increment(30);
        images.reload_images(less_images.images);
        assert_eq!(images.index(), Some(0));
        assert_eq!(images.max_viewable_index(), Some(9));
    }

    #[test]
    fn test_removing_and_replacing_images_updates_artificial_len_to_higher_user_set() {
        let mut images = dummy_paths_builder(50).with_maximum_viewable(1).build();
        images.remove_current_image();
        let less_images = dummy_paths_builder(10).build();
        images.set_actual_maximum(10);
        images.reload_images(less_images.images);
        assert_eq!(images.max_viewable(), Some(10));
    }
}
