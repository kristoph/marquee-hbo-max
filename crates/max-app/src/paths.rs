use std::path::PathBuf;

use max_api::workspace;
pub use max_api::workspace::session;

const UNPACKED_APP_ASSETS: &str = "../androidtv/out/max/resources/assets";
const HOME_CAPTURE: &str = "routes-home";
const NAVIGATION_MENU_CAPTURE: &str = "navigation-menu";

pub fn captured_home() -> PathBuf {
    workspace::capture(HOME_CAPTURE)
}

pub fn captured_navigation_menu() -> PathBuf {
    workspace::capture(NAVIGATION_MENU_CAPTURE)
}

pub fn brand_logo() -> PathBuf {
    workspace::file(UNPACKED_APP_ASSETS).join("assets/drawable/default/logo.png")
}
