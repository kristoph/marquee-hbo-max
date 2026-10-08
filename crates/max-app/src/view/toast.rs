use std::time::Duration;

use eframe::egui::{self, Pos2, Rect, Vec2};

use crate::{
    app::App,
    metrics::BODY_TEXT,
    theme::{self, regular, TEXT},
    timing,
};

const PADDING: Vec2 = Vec2::new(40.0, 24.0);
const ABOVE_WINDOW_FOOT: f32 = 60.0;
const EXPIRY_POLL: Duration = Duration::from_millis(200);

impl App {
    pub(crate) fn draw_toast(&self, ui: &mut egui::Ui, window: Rect) {
        let Some(toast) = self.toast.as_ref().filter(|toast| toast.shown_at.elapsed() < timing::TOAST) else { return };
        let painter = ui.painter();
        let text = painter.layout_no_wrap(toast.message.clone(), regular(BODY_TEXT), TEXT);
        let size = text.size() + PADDING;
        let pill = Rect::from_center_size(Pos2::new(window.center().x, window.bottom() - ABOVE_WINDOW_FOOT), size);
        painter.rect_filled(pill, size.y / 2.0, theme::TOAST);
        painter.galley(pill.center() - text.size() / 2.0, text, TEXT);
        ui.ctx().request_repaint_after(EXPIRY_POLL);
    }
}
