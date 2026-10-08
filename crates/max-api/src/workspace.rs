//! Where the project keeps what it saves between runs, wherever a program is started from.

use std::path::PathBuf;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn file(relative: &str) -> PathBuf {
    PathBuf::from(ROOT).join(relative)
}

pub fn session() -> PathBuf {
    file("capture/session.json")
}

pub fn capture(name: &str) -> PathBuf {
    file("capture").join(format!("{name}.json"))
}

pub fn image_cache() -> PathBuf {
    file("cache/images")
}
