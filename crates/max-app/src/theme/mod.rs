mod fonts;
pub mod icon;

use eframe::egui::Color32;
pub use fonts::{bold, icons, install_fonts, regular};

pub const BACKGROUND: Color32 = Color32::BLACK;
pub const PLACEHOLDER: Color32 = Color32::from_rgb(30, 30, 36);
pub const TEXT: Color32 = Color32::WHITE;
pub const DIM_TEXT: Color32 = Color32::from_rgb(204, 204, 204);
pub const RANK_NUMERAL: Color32 = Color32::from_rgb(72, 72, 82);
pub const TRANSLUCENT: Color32 = Color32::from_rgba_premultiplied(52, 52, 52, 52);
pub const LIVE_PROGRESS: Color32 = Color32::from_rgb(186, 35, 108);
pub const PANEL: Color32 = Color32::from_rgb(24, 24, 30);
pub const BROWSE_MENU: Color32 = Color32::from_rgb(12, 12, 16);
pub const TOAST: Color32 = Color32::from_rgb(40, 40, 48);

pub fn white(opacity: f32) -> Color32 {
    Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0).round() as u8)
}

pub fn from_rgba([red, green, blue, alpha]: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(red, green, blue, alpha)
}
