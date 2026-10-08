use std::path::PathBuf;

pub use max_api::storage::session;
use max_api::workspace;

const HOME_CAPTURE: &str = "routes-home";
const NAVIGATION_MENU_CAPTURE: &str = "navigation-menu";

pub fn captured_home() -> PathBuf {
    workspace::capture(HOME_CAPTURE)
}

pub fn captured_navigation_menu() -> PathBuf {
    workspace::capture(NAVIGATION_MENU_CAPTURE)
}
