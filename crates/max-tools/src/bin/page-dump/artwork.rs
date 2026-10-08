use std::sync::atomic::{AtomicUsize, Ordering};

use max_api::{
    cms::{artwork_wanted, Page},
    storage,
};
use max_media::images::{cache_size, ImageCache};

const FAILURES_SHOWN: usize = 5;

pub fn fetch(page: &Page) {
    let wanted = artwork_wanted(&page.rows);
    let tiles: usize = page.rows.iter().map(|row| row.tiles.len()).sum();
    let tiles_without_artwork: usize = page.rows.iter().map(|row| row.tiles.iter().filter(|tile| tile.artwork(row.layout()).is_none()).count()).sum();

    let failed = AtomicUsize::new(0);
    ImageCache::new(storage::image_cache()).fetch_each(&wanted, |_, result| {
        if let Err(error) = result {
            if failed.fetch_add(1, Ordering::Relaxed) < FAILURES_SHOWN {
                eprintln!("image failed: {error}");
            }
        }
    });
    println!(
        "\nArtwork: {tiles} tiles, {} distinct images, {} failed, {tiles_without_artwork} tiles without artwork; cache is {} KB",
        wanted.len(),
        failed.load(Ordering::Relaxed),
        cache_size(storage::image_cache()) / 1024,
    );
}
