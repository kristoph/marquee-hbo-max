use eframe::egui::Vec2;

use crate::{
    metrics::{
        self,
        rail::{HEADING_LOGO_TO_TILES, HEADING_TO_TILES, SPACE_BELOW, SPACE_BELOW_CAPTIONED, SPACE_BELOW_GRID_LINE, TILE_GAP, TILE_PADDING},
        top_ten, MARGIN,
    },
    model::{tiles_across, ScreenRow},
};

pub(super) struct RailGeometry {
    pub artwork: Vec2,
    pub padding: f32,
    pub numeral_room: f32,
    pub cell: Vec2,
    pub heading_height: f32,
    pub tile_gap: f32,
    pub captioned: bool,
}

impl RailGeometry {
    pub fn of(row: &ScreenRow) -> Self {
        let artwork = if row.numbered { top_ten::POSTER } else { metrics::tile_size(row.layout) };
        let captioned = metrics::has_caption(row.layout);
        let numeral_room = if row.numbered { top_ten::NUMERAL_LEFT_OF_POSTER } else { 0.0 };
        let space_below = match (row.grid_line, captioned, row.numbered) {
            (true, ..) => SPACE_BELOW_GRID_LINE,
            (false, true, _) => SPACE_BELOW_CAPTIONED,
            (false, false, true) => top_ten::SPACE_BELOW,
            (false, false, false) => SPACE_BELOW,
        };
        let has_no_heading = row.title.is_empty() && row.masthead.is_none();
        let space_under_heading = match (&row.masthead, row.numbered) {
            _ if has_no_heading => TILE_PADDING,
            (Some(_), true) => top_ten::MASTHEAD.y + top_ten::MASTHEAD_TO_TILES,
            (Some(_), false) => HEADING_LOGO_TO_TILES,
            (None, _) => HEADING_TO_TILES,
        };
        Self {
            artwork,
            padding: TILE_PADDING,
            numeral_room,
            cell: Vec2::new(numeral_room + artwork.x + TILE_PADDING * 2.0, artwork.y + TILE_PADDING + space_below),
            heading_height: space_under_heading - TILE_PADDING,
            tile_gap: if row.numbered { top_ten::TILE_GAP } else { TILE_GAP },
            captioned,
        }
    }

    pub fn left_margin(&self, row: &ScreenRow, width: f32) -> f32 {
        if !row.grid_line {
            return MARGIN;
        }
        let across = tiles_across(row.layout, width) as f32;
        let block = across * self.artwork.x + (across - 1.0) * self.tile_gap;
        ((width - block) / 2.0).max(MARGIN)
    }
}
