use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId};

const REGULAR_TYPEFACE: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
const BOLD_TYPEFACE: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Bold.ttf");
const ICON_FONT: &[u8] = include_bytes!("../../assets/fonts/FontAwesome.ttf");

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

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    for (name, font) in [(REGULAR, REGULAR_TYPEFACE), (BOLD, BOLD_TYPEFACE), (ICONS, ICON_FONT)] {
        fonts.font_data.insert(name.to_string(), Arc::new(FontData::from_static(font)));
    }
    // The toolkit's own fonts stay behind ours, for the characters ours lack.
    let fallbacks = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let ahead_of_fallbacks = |own: &str| std::iter::once(own.to_string()).chain(fallbacks.iter().cloned()).collect::<Vec<_>>();
    fonts.families.insert(FontFamily::Proportional, ahead_of_fallbacks(REGULAR));
    fonts.families.insert(FontFamily::Name(BOLD.into()), ahead_of_fallbacks(BOLD));
    fonts.families.insert(FontFamily::Name(ICONS.into()), vec![ICONS.to_string()]);
    ctx.set_fonts(fonts);
}
