mod benchmark;
mod screenshot;
mod start_requests;
pub mod walkthrough;

use std::{cell::Cell, path::PathBuf, time::Instant};

use benchmark::Benchmark;
use walkthrough::Walkthrough;

pub use walkthrough::WalkthroughMode;

#[derive(Default)]
pub struct DeveloperOptions {
    pub benchmark: bool,
    pub walkthrough: Option<WalkthroughMode>,
    pub screenshot_path: Option<String>,
    pub route_to_go_on_to: Option<String>,
    pub open_tile_menu: bool,
    pub tab_to_open: Option<usize>,
    pub open_profile_picker: bool,
    pub open_episodes: bool,
    pub clear_web_data: bool,
    pub session_to_sign_in_with: Option<PathBuf>,
}

pub struct Screenshot {
    pub path: String,
    pub due: Option<Instant>,
}

pub struct FollowingRoute {
    pub route: String,
    pub first_page_shown_at: Option<Instant>,
}

pub struct DeveloperTools {
    pub(crate) benchmark: Option<Benchmark>,
    pub(crate) tiles_drawn_before_their_artwork: Cell<u32>,
    pub(crate) walkthrough: Option<Walkthrough>,
    pub(crate) screenshot: Option<Screenshot>,
    pub(crate) following_route: Option<FollowingRoute>,
    pub(crate) open_tile_menu: bool,
    pub(crate) tab_to_open: Option<usize>,
    pub(crate) open_profile_picker: bool,
    pub(crate) open_episodes: bool,
    pub(crate) clear_web_data: bool,
    pub(crate) session_to_sign_in_with: Option<PathBuf>,
}

impl DeveloperTools {
    pub fn new(options: DeveloperOptions) -> Self {
        Self {
            benchmark: options.benchmark.then(Benchmark::new),
            tiles_drawn_before_their_artwork: Cell::new(0),
            walkthrough: options.walkthrough.map(Walkthrough::new),
            screenshot: options.screenshot_path.map(|path| Screenshot { path, due: None }),
            following_route: options.route_to_go_on_to.map(|route| FollowingRoute { route, first_page_shown_at: None }),
            open_tile_menu: options.open_tile_menu,
            tab_to_open: options.tab_to_open,
            open_profile_picker: options.open_profile_picker,
            open_episodes: options.open_episodes,
            clear_web_data: options.clear_web_data,
            session_to_sign_in_with: options.session_to_sign_in_with,
        }
    }
}
