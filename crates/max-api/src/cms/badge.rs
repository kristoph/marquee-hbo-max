use super::{
    document::{Document, Resource},
    image::Image,
};

const DEFAULT_BACKGROUND: [u8; 4] = [0, 0, 0, 178];
const DEFAULT_INK: [u8; 4] = [255, 255, 255, 255];

#[derive(Debug, Clone, PartialEq)]
pub struct Badge {
    pub label: String,
    pub background: [u8; 4],
    pub ink: [u8; 4],
    pub icon: Option<Image>,
}

impl Document {
    pub(super) fn badge(&self, item: &Resource, relationship: &str) -> Option<Badge> {
        let (overlay, icon) = self.related(item, relationship).find_map(|overlay| {
            let icon = self.first_image(overlay, "image");
            let has_words = overlay.text("label").is_some_and(|label| !label.trim().is_empty());
            (has_words || icon.is_some()).then_some((overlay, icon))
        })?;
        let style = self.first_related(overlay, "style").map(|style| &style.attributes);
        let color = |name: &str, fallback: [u8; 4]| style.and_then(|style| style[name].as_str()).and_then(parse_color).unwrap_or(fallback);
        Some(Badge {
            label: overlay.text("label").unwrap_or_default().trim().to_string(),
            background: color("backgroundColor", DEFAULT_BACKGROUND),
            ink: color("fontColor", DEFAULT_INK),
            icon,
        })
    }
}

fn parse_color(text: &str) -> Option<[u8; 4]> {
    let hex = text.strip_prefix('#').filter(|hex| hex.len() == 8)?;
    let channel = |index: usize| u8::from_str_radix(hex.get(index * 2..index * 2 + 2)?, 16).ok();
    Some([channel(0)?, channel(1)?, channel(2)?, channel(3)?])
}
