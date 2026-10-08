use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use max_api::net;

use crate::Error;

const DOWNLOAD_THREADS: usize = 16;
const LARGEST_IMAGE_BYTES: u64 = 8 * 1024 * 1024;

pub struct ImageCache {
    folder: PathBuf,
}

impl ImageCache {
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self { folder: folder.into() }
    }

    pub fn path_for(&self, source: &str, width: u32) -> PathBuf {
        let path = source.split_once("://").map_or(source, |(_, rest)| rest.split_once('/').map_or("", |(_, path)| path));
        let stem: String = path.chars().map(|character| if character.is_ascii_alphanumeric() { character } else { '_' }).collect();
        self.folder.join(format!("{stem}_w{width}.webp"))
    }

    pub fn fetch(&self, source: &str, width: u32) -> Result<PathBuf, Error> {
        let path = self.path_for(source, width);
        if path.exists() {
            return Ok(path);
        }
        let bytes = net::bytes_up_to(&resized_url(source, width), LARGEST_IMAGE_BYTES)?;
        if !is_webp(&bytes) {
            return Err(Error::NotAnImage(source.to_string()));
        }
        write_whole(&path, |partial| Ok(fs::write(partial, &bytes)?))?;
        Ok(path)
    }

    /// `on_done` runs on the download threads, with each image's index in `wanted`.
    pub fn fetch_each(&self, wanted: &[(String, u32)], on_done: impl Fn(usize, Result<PathBuf, Error>) + Sync) {
        let next = AtomicUsize::new(0);
        thread::scope(|scope| {
            for _ in 0..DOWNLOAD_THREADS.min(wanted.len()) {
                scope.spawn(|| loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some((source, width)) = wanted.get(index) else { break };
                    on_done(index, self.fetch(source, *width));
                });
            }
        });
    }
}

/// Writes under another name first, so a partial file is never mistaken for a finished one.
pub(super) fn write_whole(path: &Path, write: impl FnOnce(&Path) -> Result<(), Error>) -> Result<(), Error> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let partial = path.with_extension("part");
    write(&partial)?;
    fs::rename(&partial, path)?;
    Ok(())
}

fn resized_url(source: &str, width: u32) -> String {
    format!("{source}?f=webp&w={width}")
}

fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

pub fn cache_size(folder: impl AsRef<Path>) -> u64 {
    fs::read_dir(folder).map_or(0, |entries| entries.flatten().filter_map(|entry| entry.metadata().ok()).map(|metadata| metadata.len()).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_is_flat_and_width_specific() {
        let cache = ImageCache { folder: PathBuf::from("c") };
        let source = "https://images.example.com/oHvzM/Ieex5.jpeg";
        assert_eq!(cache.path_for(source, 400), PathBuf::from("c/oHvzM_Ieex5_jpeg_w400.webp"));
        assert_ne!(cache.path_for(source, 400), cache.path_for(source, 600));
    }

    #[test]
    fn builds_resize_url_and_recognises_webp() {
        assert_eq!(resized_url("https://i.example/a.png", 600), "https://i.example/a.png?f=webp&w=600");
        assert!(is_webp(b"RIFF\x10\x00\x00\x00WEBPVP8 "));
        assert!(!is_webp(b"{\"errors\":[{\"status\":\"400\"}]}"));
    }
}
