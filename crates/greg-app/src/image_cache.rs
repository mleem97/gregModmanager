//! Bounded decoded thumbnail cache on the UI thread.
//!
//! Slint images are thread-local presentation values. Provider I/O remains in
//! workers; unchanged local paths reuse decoded pixels across list refreshes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use greg_steam::web_api::{decode_and_cache_preview_bytes, MAX_PREVIEW_BYTES};

const MAX_ENTRIES: usize = 128;
const MAX_DECODED_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: SystemTime,
}

struct Entry {
    stamp: Stamp,
    image: slint::Image,
    bytes: usize,
    accessed: u64,
}

#[derive(Default)]
struct ImageCache {
    entries: HashMap<PathBuf, Entry>,
    bytes: usize,
    clock: u64,
    #[cfg(test)]
    decode_attempts: usize,
}

impl ImageCache {
    fn load(&mut self, path: &Path) -> slint::Image {
        let stamp = match std::fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => Stamp {
                len: metadata.len(),
                modified: match metadata.modified() {
                    Ok(modified) => modified,
                    Err(_) => return slint::Image::default(),
                },
            },
            _ => {
                self.remove(path);
                return slint::Image::default();
            }
        };
        self.clock = self.clock.wrapping_add(1);
        if let Some(entry) = self.entries.get_mut(path) {
            if entry.stamp == stamp {
                entry.accessed = self.clock;
                return entry.image.clone();
            }
        }
        self.remove(path);
        let decoded = if stamp.len > 0 && stamp.len <= MAX_PREVIEW_BYTES as u64 {
            #[cfg(test)]
            {
                self.decode_attempts += 1;
            }
            load_bounded(path)
        } else {
            None
        };
        let (image, bytes) = decoded.unwrap_or_default();
        // Cache failures too: an unchanged corrupt preview should not trigger
        // a fresh decode every time the list is searched or repainted.
        self.entries.insert(
            path.to_path_buf(),
            Entry {
                stamp,
                image: image.clone(),
                bytes,
                accessed: self.clock,
            },
        );
        self.bytes += bytes;
        while self.entries.len() > MAX_ENTRIES || self.bytes > MAX_DECODED_BYTES {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.accessed)
                .map(|(path, _)| path.clone());
            if let Some(path) = oldest {
                self.remove(&path);
            } else {
                break;
            }
        }
        image
    }

    fn remove(&mut self, path: &Path) {
        if let Some(entry) = self.entries.remove(path) {
            self.bytes = self.bytes.saturating_sub(entry.bytes);
        }
    }
}

fn load_bounded(path: &Path) -> Option<(slint::Image, usize)> {
    // Take also protects against a file growing between metadata() and read().
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_PREVIEW_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    let decoded = decode_and_cache_preview_bytes(bytes, "", 0, None).ok()?;
    let size = decoded.rgba8.len();
    let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
        &decoded.rgba8,
        decoded.width,
        decoded.height,
    );
    Some((slint::Image::from_rgba8(buffer), size))
}

/// Returns a cached thumbnail, or an empty image for missing/unsupported data.
pub fn thumbnail(path: Option<&Path>) -> slint::Image {
    thread_local! { static CACHE: RefCell<ImageCache> = RefCell::new(ImageCache::default()); }
    path.map(|path| CACHE.with(|cache| cache.borrow_mut().load(path)))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("greg-image-{name}-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn unchanged_image_decodes_once_and_changed_or_corrupt_file_invalidates() {
        let dir = dir("cache");
        let path = dir.join("preview.png");
        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]))
            .save(&path)
            .unwrap();
        let mut cache = ImageCache::default();
        assert_eq!(cache.load(&path).size().width, 2);
        assert_eq!(cache.load(&path).size().width, 2);
        assert_eq!(cache.decode_attempts, 1);
        std::fs::write(&path, b"not an image").unwrap();
        assert_eq!(cache.load(&path).size().width, 0);
        assert_eq!(cache.load(&path).size().width, 0);
        assert_eq!(
            cache.decode_attempts, 2,
            "unchanged corruption is negatively cached"
        );
        std::fs::remove_file(&path).unwrap();
        assert_eq!(cache.load(&path).size().width, 0);
        assert!(cache.entries.is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn decoded_pixel_memory_is_evicted_at_budget() {
        let dir = dir("pixel-budget");
        let source = dir.join("source.png");
        image::RgbaImage::from_pixel(512, 512, image::Rgba([1, 2, 3, 255]))
            .save(&source)
            .unwrap();
        let mut cache = ImageCache::default();
        for index in 0..40 {
            let path = dir.join(format!("image-{index}.png"));
            std::fs::copy(&source, &path).unwrap();
            assert_eq!(cache.load(&path).size().width, 512);
            assert!(cache.bytes <= MAX_DECODED_BYTES);
        }
        assert_eq!(cache.entries.len(), MAX_DECODED_BYTES / (512 * 512 * 4));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn oversized_files_are_rejected_before_read_and_cache_entries_are_bounded() {
        let dir = dir("limits");
        let path = dir.join("huge.png");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_PREVIEW_BYTES as u64 + 1).unwrap();
        let mut cache = ImageCache::default();
        assert_eq!(cache.load(&path).size().width, 0);
        assert_eq!(cache.decode_attempts, 0);
        for index in 0..MAX_ENTRIES + 10 {
            let path = dir.join(format!("bad-{index}.png"));
            std::fs::write(&path, b"bad").unwrap();
            cache.load(&path);
        }
        assert_eq!(cache.entries.len(), MAX_ENTRIES);
        assert!(cache.bytes <= MAX_DECODED_BYTES);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
