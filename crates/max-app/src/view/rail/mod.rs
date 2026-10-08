mod caption;
mod geometry;
mod heading;
mod overlays;
mod tile;

use eframe::egui::{self, scroll_area::ScrollBarVisibility, Image, Rect, ScrollArea, Sense, Vec2};
use geometry::RailGeometry;
use tile::TileCell;

use crate::{
    app::App,
    intent::Intent,
    metrics::{
        header, preload,
        rail::{HEADING_TO_TILES, PLACEHOLDER_TILES, TILE_CORNER_RADIUS, TILE_GAP},
        MARGIN,
    },
    model::{ScreenRow, Selection},
    theme::PLACEHOLDER,
};

const SPACE_KEPT_ABOVE_REVEALED_TILE: f32 = header::HEIGHT + HEADING_TO_TILES + 20.0;

impl App {
    pub(crate) fn draw_rail(&self, ui: &mut egui::Ui, row_index: usize, row: &ScreenRow, pointer_moved: bool, intent: &mut Intent) {
        let geometry = RailGeometry::of(row);
        let bounds = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), geometry.heading_height + geometry.cell.y));
        let revealing = self.page.is_revealing_selection() && self.page.selected.row == row_index;
        if !ui.is_rect_visible(bounds) && !revealing {
            ui.allocate_exact_size(bounds.size(), Sense::hover());
            preload_row_about_to_scroll_into_view(ui, row, &geometry, bounds);
            return;
        }
        heading::paint(ui, row, geometry.heading_height);
        if row.pending {
            return paint_placeholders(ui, &geometry);
        }

        let mut revealed = None;
        ScrollArea::horizontal()
            .id_salt(("rail", self.navigation.generation, self.page.tabs.as_ref().map(|tabs| tabs.selected), row_index))
            .auto_shrink([false, true])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = geometry.tile_gap - geometry.padding * 2.0;
                    ui.add_space(geometry.left_margin(row, bounds.width()) - geometry.padding);
                    for (column, tile) in row.tiles.iter().enumerate() {
                        let at = Selection::new(row_index, column);
                        let (cell, response) = ui.allocate_exact_size(geometry.cell, Sense::click());
                        self.collect_tile_clicks(&response, cell, tile, at, pointer_moved, intent);
                        if self.page.selected == at && self.page.is_revealing_selection() {
                            let view =
                                Rect::from_min_max(cell.min - Vec2::new(MARGIN, SPACE_KEPT_ABOVE_REVEALED_TILE), cell.max + Vec2::new(MARGIN, 0.0));
                            ui.scroll_to_rect(view, None);
                            revealed = Some(view);
                        }
                        if ui.is_rect_visible(cell) {
                            self.draw_tile(ui, row, &geometry, TileCell { tile, at, bounds: cell, response: &response }, intent);
                        } else if ui.clip_rect().expand2(Vec2::new(preload::SIDEWAYS, 0.0)).intersects(cell) {
                            preload(ui, tile.artwork.as_ref().and_then(|artwork| artwork.uri()), geometry.artwork);
                        }
                    }
                    ui.add_space(MARGIN);
                });
            });
        // A scroll area swallows scroll requests made inside it on both axes, so the request
        // for the page's vertical scroll is repeated out here.
        if let Some(view) = revealed {
            ui.scroll_to_rect(view, None);
        }
    }
}

fn preload_row_about_to_scroll_into_view(ui: &egui::Ui, row: &ScreenRow, geometry: &RailGeometry, bounds: Rect) {
    if !ui.clip_rect().expand2(Vec2::new(0.0, preload::ABOVE_AND_BELOW)).intersects(bounds) {
        return;
    }
    let tiles_across = (bounds.width() / (geometry.artwork.x + TILE_GAP)).ceil() as usize + 1;
    for tile in row.tiles.iter().take(tiles_across) {
        preload(ui, tile.artwork.as_ref().and_then(|artwork| artwork.uri()), geometry.artwork);
    }
}

fn preload(ui: &egui::Ui, uri: Option<&str>, size: Vec2) {
    if let Some(uri) = uri {
        let _ = Image::new(uri).load_for_size(ui.ctx(), size);
    }
}

fn paint_placeholders(ui: &mut egui::Ui, geometry: &RailGeometry) {
    let (strip, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), geometry.cell.y), Sense::hover());
    for place in 0..PLACEHOLDER_TILES {
        let corner = strip.min + Vec2::new(MARGIN + place as f32 * (geometry.artwork.x + TILE_GAP), geometry.padding);
        ui.painter().rect_filled(Rect::from_min_size(corner, geometry.artwork), TILE_CORNER_RADIUS, PLACEHOLDER);
    }
}
