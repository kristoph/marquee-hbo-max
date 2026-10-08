//! Where the project's own files are, for the tools a developer runs from a checkout.

use std::path::PathBuf;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn file(relative: &str) -> PathBuf {
    PathBuf::from(ROOT).join(relative)
}

pub fn capture(name: &str) -> PathBuf {
    file("capture").join(format!("{name}.json"))
}
