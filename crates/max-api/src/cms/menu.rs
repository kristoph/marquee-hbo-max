use serde_json::Value;

use super::document::{text_at, Document, Resource};

const ROUTE_PARAMETERS: [(&str, &str); 3] = [("startPosition", "pos"), ("intent", "intent"), ("editId", "editId")];

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    Go { label: String, context: String, route: String },
    MyList { url: String, listed: bool, add: String, remove: String, added: String, removed: String },
    Rate { url: String, title: String, options: Vec<RatingOption>, chosen: Option<String> },
    Remove { url: String, label: String, done: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatingOption {
    pub value: String,
    pub label: String,
    pub chosen_label: String,
}

impl Document {
    pub(super) fn menu(&self, item: &Resource) -> Vec<MenuAction> {
        let mut menu: Vec<MenuAction> = Vec::new();
        for action in self.related(item, "userActions") {
            let template = self.first_related(action, "actionTemplate").map(|template| &template.attributes);
            let context = action.text("context").unwrap_or_default();
            let entry = match (action.text("actionType").unwrap_or_default(), context) {
                ("navigate", _) => self.navigation_entry(action, context),
                ("toggle", "myList") => my_list_entry(action, template),
                ("select", "explicitSignals") => rating_entry(action, template),
                ("remove", _) => removal_entry(action, template),
                _ => None,
            };
            menu.extend(entry.filter(|entry| !menu.contains(entry)));
        }
        menu
    }

    fn navigation_entry(&self, action: &Resource, context: &str) -> Option<MenuAction> {
        let mut route = self.action_route(action)?;
        let parameters = &action.attributes["elements"]["routeParams"];
        let query: Vec<String> = ROUTE_PARAMETERS
            .iter()
            .filter_map(|(key, name)| match &parameters[*key] {
                Value::String(text) => Some(format!("{name}={text}")),
                Value::Number(number) => Some(format!("{name}={number}")),
                _ => None,
            })
            .collect();
        if !query.is_empty() {
            route = format!("{route}?{}", query.join("&"));
        }
        Some(MenuAction::Go { label: text_at(&action.attributes, &["elements", "label", "label"])?, context: context.to_string(), route })
    }
}

/// The template's "on" state is the offer to add and its "off" state the offer to remove; the
/// action names the state currently on offer.
fn my_list_entry(action: &Resource, template: Option<&Value>) -> Option<MenuAction> {
    let template = template?;
    Some(MenuAction::MyList {
        url: action.owned_text("url")?,
        listed: action.text("initialState") == Some("off"),
        add: text_at(template, &["on", "elements", "label", "label"])?,
        remove: text_at(template, &["off", "elements", "label", "label"])?,
        added: text_at(template, &["on", "elements", "feedback", "label"]).unwrap_or_default(),
        removed: text_at(template, &["off", "elements", "feedback", "label"]).unwrap_or_default(),
    })
}

fn rating_entry(action: &Resource, template: Option<&Value>) -> Option<MenuAction> {
    let template = template?;
    let options = template["options"]
        .as_array()?
        .iter()
        .filter_map(|option| {
            Some(RatingOption {
                value: option["value"].as_str()?.to_string(),
                label: text_at(option, &["on", "elements", "label", "label"])?,
                chosen_label: text_at(option, &["off", "elements", "label", "label"])?,
            })
        })
        .collect();
    Some(MenuAction::Rate {
        url: action.owned_text("url")?,
        title: text_at(template, &["elements", "title", "label"]).unwrap_or_default(),
        options,
        chosen: action.owned_text("selected"),
    })
}

fn removal_entry(action: &Resource, template: Option<&Value>) -> Option<MenuAction> {
    let template = template?;
    Some(MenuAction::Remove {
        url: action.owned_text("url")?,
        label: text_at(template, &["elements", "label", "label"])?,
        done: text_at(template, &["elements", "feedback", "label"]).unwrap_or_default(),
    })
}
