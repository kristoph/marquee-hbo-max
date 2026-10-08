use super::{
    action::Action,
    badge::Badge,
    document::{Document, Resource},
    image::Image,
    layout::Layout,
    menu::MenuAction,
};

const TITLE_LOGO_KINDS: [&str; 3] = ["logo-left", "content-logo-polychromatic", "logo-centered"];
const TARGETS: [(&str, TileKind); 6] = [
    ("show", TileKind::Show),
    ("video", TileKind::Video),
    ("link", TileKind::Link),
    ("taxonomyNode", TileKind::Category),
    ("collection", TileKind::Collection),
    ("view", TileKind::View),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Show,
    Video,
    Link,
    Category,
    Collection,
    Channel,
    View,
}

#[derive(Debug)]
pub struct Tile {
    pub kind: TileKind,
    pub title: String,
    pub route: Option<String>,
    pub images: Vec<Image>,
    pub show_images: Vec<Image>,
    pub detail: TileDetail,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TileDetail {
    pub preview_edit_id: Option<String>,
    pub season_and_episode: Option<(u32, u32)>,
    pub facts: Vec<String>,
    pub description: Option<String>,
    pub secondary_title: Option<String>,
    pub rating: Option<String>,
    pub genres: Vec<String>,
    pub badge: Option<Badge>,
    pub banner: Option<Badge>,
    pub highlight: Option<String>,
    pub progress: Option<f32>,
    pub live: bool,
    pub actions: Vec<Action>,
    pub menu: Vec<MenuAction>,
}

impl Tile {
    pub fn logo(&self) -> Option<&Image> {
        TITLE_LOGO_KINDS.iter().find_map(|kind| self.images.iter().find(|image| image.kind == *kind))
    }

    pub fn artwork(&self, layout: Layout) -> Option<&Image> {
        layout.pick_artwork(&self.images)
    }

    pub fn show_artwork(&self, layout: Layout) -> Option<&Image> {
        layout.pick_artwork(&self.show_images)
    }
}

impl Document {
    pub(super) fn tile(&self, item: &Resource) -> Option<Tile> {
        let (kind, target) = TARGETS.iter().find_map(|(relationship, kind)| Some((*kind, self.first_related(item, relationship)?)))?;
        if kind == TileKind::Collection {
            if let Some(channel) = self.channel_tile(target) {
                return Some(channel);
            }
        }
        let route = self
            .first_related(item, "defaultAction")
            .and_then(|action| self.action_route(action))
            .or_else(|| self.first_related(target, "routes").and_then(|route| route.owned_text("url")));
        let show_images = self.first_related(target, "show").map(|show| self.images_of(show)).unwrap_or_default();
        Some(Tile { kind, title: self.title_of(kind, target), route, images: self.images_of(target), show_images, detail: self.detail(item, target) })
    }

    /// A link's own name is an internal alias; what it links to carries the name to display.
    fn title_of(&self, kind: TileKind, target: &Resource) -> String {
        let named = match kind {
            TileKind::Link => self.first_related(target, "linkedContent").unwrap_or(target),
            _ => target,
        };
        ["title", "name"]
            .iter()
            .find_map(|attribute| named.text(attribute))
            .or_else(|| target.attributes["fallbacks"]["title"].as_str())
            .unwrap_or_default()
            .to_string()
    }

    fn detail(&self, item: &Resource, target: &Resource) -> TileDetail {
        let actions = self.actions(item);
        let number = |name: &str| target.attributes[name].as_u64().map(|number| number as u32);
        let rating = self.labels(target, "ratings").into_iter().next();
        TileDetail {
            preview_edit_id: self
                .first_related(target, "shortPreviewVideo")
                .and_then(|preview| preview.related_keys("edit").first())
                .map(|edit| edit.id.clone()),
            season_and_episode: number("seasonNumber").zip(number("episodeNumber")),
            facts: rating.clone().into_iter().chain(seasons_or_year(target)).collect(),
            description: target.owned_text("description"),
            secondary_title: target.owned_text("secondaryTitle"),
            rating,
            genres: self.labels(target, "txGenres"),
            badge: self.badge(item, "badges"),
            banner: self.badge(item, "banners"),
            highlight: self.labels(item, "highlights").into_iter().next(),
            progress: actions.iter().find_map(|action| action.progress),
            live: actions.iter().any(|action| action.live),
            actions,
            menu: self.menu(item),
        }
    }

    /// In the Channels rail a tile is a nested collection whose one item is what is airing now.
    fn channel_tile(&self, nested: &Resource) -> Option<Tile> {
        let item = self.first_related(nested, "items")?;
        let airing = self.first_related(item, "airing")?;
        let channel = self.first_related(airing, "distributionChannel");
        let programme = match (airing.text("showName"), airing.text("name")) {
            (Some(show), Some(episode)) => Some(format!("{show}: {episode}")),
            (show, episode) => show.or(episode).map(str::to_string),
        };
        let title = channel.and_then(|channel| channel.owned_text("name")).or_else(|| programme.clone())?;
        let mut images = channel.map(|channel| self.images_of(channel)).unwrap_or_default();
        images.extend(self.first_related(airing, "video").map(|video| self.images_of(video)).unwrap_or_default());
        let actions = self.actions(item);
        Some(Tile {
            kind: TileKind::Channel,
            title,
            route: actions.first().and_then(|action| action.route.clone()),
            images,
            show_images: Vec::new(),
            detail: TileDetail {
                description: airing.owned_text("description"),
                secondary_title: programme,
                rating: self.labels(airing, "ratings").into_iter().next(),
                badge: self.badge(item, "badges"),
                actions,
                menu: self.menu(item),
                ..TileDetail::default()
            },
        })
    }
}

fn seasons_or_year(target: &Resource) -> Option<String> {
    let seasons = target.related_keys("seasons").len();
    match target.text("showType") {
        Some("SERIES" | "MINISERIES" | "TOPICAL") if seasons > 0 => Some(format!("{seasons} Season{}", if seasons == 1 { "" } else { "s" })),
        _ => ["premiereDate", "airDate"].iter().find_map(|date| target.text(date)?.get(..4)).map(str::to_string),
    }
}
