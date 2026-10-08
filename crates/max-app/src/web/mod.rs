pub mod cookies;

use eframe::egui::Rect;
use wry::dpi::{LogicalPosition, LogicalSize};

pub const BLACK: (u8, u8, u8, u8) = (0, 0, 0, 255);

pub fn bounds(area: Rect) -> wry::Rect {
    wry::Rect { position: LogicalPosition::new(area.left(), area.top()).into(), size: LogicalSize::new(area.width(), area.height()).into() }
}

pub fn without_query(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or_default()
}
