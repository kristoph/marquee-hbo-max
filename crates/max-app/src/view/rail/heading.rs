use eframe::egui::{self, Align2, Image, Rect, Sense, Vec2};

use crate::{
    metrics::{rail::HEADING_LOGO_HEIGHT, top_ten, HEADING_TEXT, MARGIN},
    model::ScreenRow,
    theme::{bold, TEXT},
};

/// A row headed by a logo shows nothing until the logo arrives, rather than flash its title
/// as text first.
pub(super) fn paint(ui: &mut egui::Ui, row: &ScreenRow, height: f32) {
    let (heading, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let corner = heading.left_top() + Vec2::new(MARGIN, 0.0);
    match &row.masthead {
        None => {
            ui.painter().text(corner, Align2::LEFT_TOP, &row.title, bold(HEADING_TEXT), TEXT);
        }
        Some(masthead) => {
            let logo_height = if row.numbered { top_ten::MASTHEAD.y } else { HEADING_LOGO_HEIGHT };
            if let Some(uri) = masthead.uri() {
                Image::new(uri).paint_at(ui, Rect::from_min_size(corner, Vec2::new(logo_height * masthead.aspect, logo_height)));
            }
        }
    }
}
