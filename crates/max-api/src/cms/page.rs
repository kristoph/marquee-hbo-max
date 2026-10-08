use super::{
    document::{Document, Resource},
    image::Image,
    layout::Layout,
    tile::Tile,
};

const TAB_GROUP_COMPONENT: &str = "tab-group";
const NUMBERED_COMPONENT: &str = "numbered";
pub const CONTINUE_WATCHING_TEMPLATE: &str = "continue-watching";

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub rows: Vec<Row>,
}

#[derive(Debug)]
pub struct Row {
    pub id: String,
    pub title: String,
    pub alias: String,
    pub component: String,
    pub template: Option<String>,
    pub deferred: bool,
    pub mandatory_parameters: Option<String>,
    pub masthead: Option<Image>,
    pub rank_images: Vec<Image>,
    pub rank_images_selected: Vec<Image>,
    pub grid: bool,
    pub use_show_artwork: bool,
    pub filters: Vec<Filter>,
    pub tabs: Vec<Tab>,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub label: String,
    pub parameter: String,
}

#[derive(Debug)]
pub struct Tab {
    pub title: String,
    pub rows: Vec<Row>,
}

impl Row {
    pub fn is_numbered(&self) -> bool {
        self.component == NUMBERED_COMPONENT
    }

    pub fn resumes_watching(&self) -> bool {
        self.template.as_deref() == Some(CONTINUE_WATCHING_TEMPLATE)
    }

    pub fn layout(&self) -> Layout {
        Layout::of_component(&self.component)
    }
}

impl Document {
    pub fn page(&self) -> Option<Page> {
        let page = match self.root.kind.as_str() {
            "page" => &self.root,
            _ => self.first_related(&self.root, "target").filter(|target| target.kind == "page")?,
        };
        let rows = self.nested_collections(page).map(|collection| self.row(collection)).collect();
        Some(Page { title: page.text("title").unwrap_or_default().to_string(), rows })
    }

    pub fn collection(&self) -> Option<Row> {
        (self.root.kind == "collection").then(|| self.row(&self.root))
    }

    fn nested_collections<'document>(&'document self, from: &'document Resource) -> impl Iterator<Item = &'document Resource> + 'document {
        self.related(from, "items").filter_map(|item| self.first_related(item, "collection"))
    }

    fn row(&self, collection: &Resource) -> Row {
        let component = &collection.attributes["component"];
        let is_tab_group = component["id"].as_str() == Some(TAB_GROUP_COMPONENT);
        let flag = |name: &str| component["customAttributes"][name].as_bool().unwrap_or(false);
        Row {
            id: collection.id.clone(),
            title: collection.text("title").unwrap_or_default().to_string(),
            alias: collection.text("alias").unwrap_or_default().to_string(),
            component: component["id"].as_str().unwrap_or_default().to_string(),
            template: component["templateId"].as_str().map(str::to_string),
            deferred: collection.attributes["async"].as_bool().unwrap_or(false),
            mandatory_parameters: component["mandatoryParams"].as_str().map(str::to_string),
            masthead: self.first_image(collection, "leftLogoImage"),
            rank_images: self.supporting_images(collection, "defaultSupportingImages"),
            rank_images_selected: self.supporting_images(collection, "alternateSupportingImages"),
            grid: flag("grid"),
            use_show_artwork: flag("useShowArt"),
            filters: filters_of(component),
            tabs: if is_tab_group { self.tabs(collection) } else { Vec::new() },
            tiles: if is_tab_group { Vec::new() } else { self.related(collection, "items").filter_map(|item| self.tile(item)).collect() },
        }
    }

    fn supporting_images(&self, collection: &Resource, relationship: &str) -> Vec<Image> {
        self.first_related(collection, relationship)
            .into_iter()
            .flat_map(|images| self.related(images, "items"))
            .filter_map(|item| self.first_image(item, "image"))
            .collect()
    }

    fn tabs(&self, group: &Resource) -> Vec<Tab> {
        self.nested_collections(group)
            .map(|tab| Tab {
                title: tab.text("title").unwrap_or_default().to_string(),
                rows: self.nested_collections(tab).map(|row| self.row(row)).collect(),
            })
            .collect()
    }
}

fn filters_of(component: &serde_json::Value) -> Vec<Filter> {
    component["filters"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|filter| filter["options"].as_array().into_iter().flatten())
        .filter_map(|option| Some(Filter { label: option["value"].as_str()?.to_string(), parameter: option["parameter"].as_str()?.to_string() }))
        .collect()
}
