use std::{fs, path::Path};

use eframe::egui::Vec2;
use max_api::{
    client::Chrome,
    cms::{Document, NavigationItem, ICON_WIDTH},
};
use max_media::images::scaled_copy;

use crate::{
    artwork,
    metrics::header::{AVATAR, ICON, IMAGE_PIXELS_PER_POINT, LOGO},
    paths,
    service::Service,
    Failure,
};

#[derive(Default)]
pub struct ScreenChrome {
    pub navigation: Vec<NavigationItem>,
    pub logo: Option<String>,
    pub search_icon: Option<String>,
    pub my_stuff_icon: Option<String>,
    pub avatar: Option<String>,
}

pub fn load_chrome(service: &Service) -> Result<ScreenChrome, Failure> {
    let chrome = match service {
        Service::Live(client) => client.chrome()?,
        Service::Captured => Chrome::from_navigation_menu(&Document::parse(&fs::read_to_string(paths::captured_navigation_menu())?)?),
        Service::SignedOut => return Err("not signed in".into()),
    };
    let prepared = |source: &Option<String>, size: Vec2| prepare_image(source.as_ref()?, ICON_WIDTH, size);
    Ok(ScreenChrome {
        logo: prepared(&chrome.logo, LOGO).or_else(|| file_uri(&paths::brand_logo())),
        search_icon: prepared(&chrome.search_icon, Vec2::splat(ICON)),
        my_stuff_icon: prepared(&chrome.my_stuff_icon, Vec2::splat(ICON)),
        avatar: prepared(&chrome.avatar, Vec2::splat(AVATAR)),
        navigation: chrome.navigation,
    })
}

pub fn prepare_image(source: &str, fetch_width: u32, size: Vec2) -> Option<String> {
    let fetched = artwork::cache().fetch(source, fetch_width).map_err(|error| log::warn!("icon failed: {error}")).ok()?;
    let pixels = size * IMAGE_PIXELS_PER_POINT;
    let scaled = scaled_copy(&fetched, pixels.x.round() as u32, pixels.y.round() as u32);
    file_uri(&scaled.map_err(|error| log::warn!("icon failed: {error}")).unwrap_or(fetched))
}

fn file_uri(path: &Path) -> Option<String> {
    Some(format!("file://{}", path.canonicalize().ok()?.display()))
}
