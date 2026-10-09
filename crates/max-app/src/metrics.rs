//! Sizes in points, measured from the original web player in a 1600-point-wide window. They
//! do not grow with the window; a larger window shows more of the page.

use eframe::egui::Vec2;
use max_api::cms::Layout;

pub const MARGIN: f32 = 30.0;
pub const CORNER_RADIUS: f32 = 8.0;
/// A wider window shows the page centred at this width, with nothing beside it.
pub const MAX_CONTENT_WIDTH: f32 = 2560.0;
pub const HEADING_TEXT: f32 = 16.667;
pub const BODY_TEXT: f32 = 16.667;
pub const SMALL_TEXT: f32 = 13.889;
pub const SPACE_ABOVE_FIRST_RAIL: f32 = header::HEIGHT + 30.0;
pub const SPACE_BELOW_PAGE: f32 = 80.0;

pub mod header {
    use super::{Vec2, MARGIN};

    pub const ICON: f32 = 28.0;
    pub const AVATAR: f32 = 32.0;
    pub const LOGO: Vec2 = Vec2::new(44.0, 32.0);
    pub const HEIGHT: f32 = ICON + MARGIN * 2.0;
    pub const IMAGE_PIXELS_PER_POINT: f32 = 2.0;
    pub const BROWSE_MENU_WIDTH: f32 = 340.0;
    pub const BROWSE_MENU_LINE: f32 = 56.0;
}

pub mod hero {
    use super::Vec2;

    pub const FULL_HEIGHT: f32 = 900.0;
    pub const TEXT_WIDTH: f32 = 480.0;
    pub const FRACTION_ABOVE_FIRST_RAIL: f32 = 0.849;
    pub const BUTTONS_ABOVE_FIRST_RAIL: f32 = 176.0;
    pub const BUTTON_HEIGHT: f32 = 52.0;
    pub const LOGO_SPACE: Vec2 = Vec2::new(480.0, 190.0);
    pub const DOT_RADIUS: f32 = 6.0;
    pub const DOT_SPACING: f32 = 30.0;
    pub const DOTS_ABOVE_FIRST_RAIL: f32 = 40.0;
    pub const PROGRESS_RING_RADIUS: f32 = 9.0;
    pub const PROGRESS_RING_WIDTH: f32 = 3.5;
    pub const DRAG_DISTANCE_FOR_A_STEP: f32 = 60.0;
}

pub mod rail {
    pub const HEADING_TO_TILES: f32 = 34.0;
    pub const HEADING_LOGO_HEIGHT: f32 = 32.0;
    pub const HEADING_LOGO_TO_TILES: f32 = HEADING_LOGO_HEIGHT + 14.0;
    pub const TILE_GAP: f32 = 32.0;
    pub const TILE_PADDING: f32 = 16.0;
    pub const TILE_CORNER_RADIUS: f32 = 1.0;
    pub const SELECTION_RING_RADIUS: f32 = 2.0;
    pub const SELECTED_SCALE: f32 = 1.05;
    pub const BADGE_HEIGHT: f32 = 24.0;
    pub const BADGE_ICON_HEIGHT: f32 = 13.0;
    pub const BANNER_HEIGHT: f32 = 24.0;
    pub const PROGRESS_BAR_HEIGHT: f32 = 4.0;
    pub const MORE_BUTTON: f32 = 32.0;
    pub const SPACE_BELOW: f32 = 30.0;
    pub const SPACE_BELOW_CAPTIONED: f32 = 75.0;
    pub const SPACE_BELOW_GRID_LINE: f32 = 65.0;
    pub const PLACEHOLDER_TILES: usize = 8;
}

pub mod top_ten {
    use super::Vec2;

    pub const MASTHEAD: Vec2 = Vec2::new(480.0, 144.0);
    pub const MASTHEAD_TO_TILES: f32 = 20.0;
    pub const POSTER: Vec2 = Vec2::new(170.0, 255.0);
    pub const NUMERAL: Vec2 = Vec2::new(92.0, 100.0);
    pub const NUMERAL_LEFT_OF_POSTER: f32 = 81.0;
    pub const NUMERAL_ABOVE_POSTER_FOOT: f32 = 3.0;
    pub const TILE_GAP: f32 = 10.0;
    pub const SPACE_BELOW: f32 = 68.0;
}

pub mod search {
    pub const BAR_HEIGHT: f32 = 64.0;
    pub const BAR_TO_TOPICS: f32 = 24.0;
    pub const TOPIC_HEIGHT: f32 = 40.0;
    pub const TOPICS_TO_ROWS: f32 = 44.0;
}

pub mod tile_menu {
    pub const WIDTH: f32 = 300.0;
    pub const LINE_HEIGHT: f32 = 44.0;
    pub const PROGRESS_WIDTH: f32 = 120.0;
}

pub mod profiles {
    pub const AVATAR: f32 = 150.0;
    pub const AVATAR_GAP: f32 = 48.0;
    pub const TITLE_TEXT: f32 = 40.0;
    pub const NAME_TEXT: f32 = 20.0;
    pub const BUTTON_HEIGHT: f32 = 48.0;
}

pub mod preload {
    pub const ABOVE_AND_BELOW: f32 = 2000.0;
    pub const SIDEWAYS: f32 = 900.0;
}

pub fn tile_size(layout: Layout) -> Vec2 {
    match layout {
        Layout::Hero => Vec2::ZERO,
        Layout::Showcase => Vec2::new(480.0, 270.0),
        Layout::Poster => Vec2::new(230.0, 345.0),
        Layout::Square => Vec2::new(230.0, 230.0),
        Layout::Landscape | Layout::Other => Vec2::new(355.0, 200.0),
    }
}

pub fn has_caption(layout: Layout) -> bool {
    matches!(layout, Layout::Landscape | Layout::Showcase | Layout::Other)
}
