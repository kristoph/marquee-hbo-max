use super::document::Document;

const CATEGORIES_ALIAS_SUFFIX: &str = "menu-categories";

#[derive(Debug, Clone, PartialEq)]
pub struct NavigationItem {
    pub label: String,
    pub route: Option<String>,
}

impl Document {
    pub fn navigation_items(&self) -> Vec<NavigationItem> {
        let categories = self
            .included()
            .find(|resource| resource.kind == "collection" && resource.text("alias").is_some_and(|alias| alias.ends_with(CATEGORIES_ALIAS_SUFFIX)));
        let Some(categories) = categories else { return Vec::new() };
        self.related(categories, "items")
            .filter_map(|item| self.first_related(item, "collection"))
            .filter_map(|entry| {
                let title = entry.text("title").or_else(|| entry.text("name"))?;
                let route = self
                    .related(entry, "items")
                    .filter_map(|item| self.first_related(item, "link"))
                    .find_map(|link| self.first_related(link, "linkedContentRoutes"))
                    .and_then(|route| route.owned_text("url"));
                Some(NavigationItem { label: label_of(title), route })
            })
            .collect()
    }
}

/// An entry drawn as an image has only an alias for a name, such as `wemo-nav-item-hbo`.
fn label_of(title: &str) -> String {
    match title.rsplit_once("-item-") {
        Some((_, name)) => name.to_uppercase(),
        None => title.to_string(),
    }
}
