use super::document::{Document, Resource};

pub const LOGO_WIDTH: u32 = 800;
pub const ICON_WIDTH: u32 = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub kind: String,
    pub source: String,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub(super) fn from_resource(resource: &Resource) -> Option<Self> {
        Some(Self {
            kind: resource.text("kind").unwrap_or_default().to_string(),
            source: resource.owned_text("src")?,
            width: resource.attributes["width"].as_u64()? as u32,
            height: resource.attributes["height"].as_u64()? as u32,
        })
    }
}

impl Document {
    pub fn image_named(&self, name: &str) -> Option<Image> {
        self.included().filter(|resource| resource.kind == "image" && resource.text("name") == Some(name)).find_map(Image::from_resource)
    }

    pub(super) fn images_of(&self, owner: &Resource) -> Vec<Image> {
        self.related(owner, "images").filter_map(Image::from_resource).collect()
    }

    pub(super) fn first_image(&self, owner: &Resource, relationship: &str) -> Option<Image> {
        self.first_related(owner, relationship).and_then(Image::from_resource)
    }
}
