use std::{
    fs,
    path::Path,
    sync::{Arc, OnceLock},
};

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId};

use crate::paths;

const REGULAR: &str = "regular";
const BOLD: &str = "bold";
const ICONS: &str = "icons";

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}

pub fn icons(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(ICONS.into()))
}

static ICONS_INSTALLED: OnceLock<bool> = OnceLock::new();

/// Without the icon font the app writes words where it would draw a glyph.
pub fn has_icons() -> bool {
    ICONS_INSTALLED.get().copied().unwrap_or(false)
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, path: &Path| {
        fs::read(path).is_ok_and(|bytes| fonts.font_data.insert(name.to_string(), Arc::new(FontData::from_owned(bytes))).is_none())
    };
    let has_icons = add(ICONS, &paths::icon_font());
    let has_regular = add(REGULAR, &paths::typeface("yi_Handset Sans UI-Regular.ttf"));
    let has_bold = add(BOLD, &paths::typeface("yi_Handset Sans UI-Bold.ttf"));

    let fallbacks = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let family = |own: Option<&str>| own.map(str::to_string).into_iter().chain(fallbacks.iter().cloned()).collect::<Vec<_>>();
    fonts.families.insert(FontFamily::Proportional, family(has_regular.then_some(REGULAR)));
    fonts.families.insert(FontFamily::Name(BOLD.into()), family(has_bold.then_some(BOLD)));
    if has_icons {
        fonts.families.insert(FontFamily::Name(ICONS.into()), vec![ICONS.to_string()]);
    }
    ctx.set_fonts(fonts);
    let _ = ICONS_INSTALLED.set(has_icons);
}
