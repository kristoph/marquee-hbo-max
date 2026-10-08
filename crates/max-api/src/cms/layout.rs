use super::image::Image;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Hero,
    Poster,
    Landscape,
    Showcase,
    Square,
    Other,
}

impl Layout {
    pub(super) fn of_component(component: &str) -> Self {
        match component {
            "hero" => Layout::Hero,
            "2-3" | "numbered" => Layout::Poster,
            "16-9" | "multilevel" => Layout::Landscape,
            "showcase" => Layout::Showcase,
            "1-1" => Layout::Square,
            _ => Layout::Other,
        }
    }

    fn preferred_image_kinds(self) -> &'static [&'static str] {
        match self {
            Layout::Poster => &["poster-with-logo", "box-art"],
            Layout::Landscape => &["cover-artwork-horizontal", "default"],
            Layout::Hero | Layout::Showcase | Layout::Other => &["default", "cover-artwork-horizontal"],
            Layout::Square => &["cover-artwork-square", "cover-artwork"],
        }
    }

    fn aspect(self) -> f32 {
        match self {
            Layout::Poster => 2.0 / 3.0,
            Layout::Square => 1.0,
            _ => 16.0 / 9.0,
        }
    }

    /// The image service resizes only to 200, 400, 600, 800, 1000, 1280 and 1920 pixels wide.
    pub fn fetch_width(self) -> u32 {
        match self {
            Layout::Hero => 1920,
            Layout::Showcase => 1000,
            Layout::Landscape | Layout::Other => 600,
            Layout::Poster | Layout::Square => 400,
        }
    }

    pub(super) fn pick_artwork(self, images: &[Image]) -> Option<&Image> {
        let preferred = self.preferred_image_kinds().iter().find_map(|kind| images.iter().find(|image| image.kind == *kind));
        preferred.or_else(|| {
            let distance_from_shape = |image: &&Image| (image.aspect() - self.aspect()).abs();
            images
                .iter()
                .filter(|image| !is_title_logo(&image.kind))
                .min_by(|first, second| distance_from_shape(first).total_cmp(&distance_from_shape(second)))
        })
    }
}

/// `poster-with-logo` is artwork; the kinds matched here are transparent title treatments.
fn is_title_logo(kind: &str) -> bool {
    kind == "logo" || kind.starts_with("logo-") || kind.starts_with("content-logo")
}
