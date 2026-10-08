use std::sync::OnceLock;

use max_api::workspace;
use max_media::images::ImageCache;

pub fn cache() -> &'static ImageCache {
    static CACHE: OnceLock<ImageCache> = OnceLock::new();
    CACHE.get_or_init(|| ImageCache::new(workspace::image_cache()))
}
