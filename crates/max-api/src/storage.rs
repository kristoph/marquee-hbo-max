//! Where the app keeps what it saves between runs: the folders the system gives each person
//! for an app's own data and for what it can always fetch again.

use std::{env, path::PathBuf};

const APP_FOLDER: &str = "Marquee for HBO Max";

fn library(kind: &str) -> PathBuf {
    env::var_os("HOME").map_or_else(env::temp_dir, PathBuf::from).join("Library").join(kind).join(APP_FOLDER)
}

pub fn session() -> PathBuf {
    library("Application Support").join("session.json")
}

pub fn image_cache() -> PathBuf {
    library("Caches").join("images")
}
