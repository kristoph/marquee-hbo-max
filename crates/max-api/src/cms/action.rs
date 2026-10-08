use super::document::{Document, Resource};

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub label: String,
    pub context: String,
    pub progress: Option<f32>,
    pub live: bool,
    pub route: Option<String>,
}

impl Action {
    pub fn plays(&self) -> bool {
        matches!(self.context.as_str(), "play" | "resume")
    }
}

impl Document {
    pub(super) fn action_route(&self, action: &Resource) -> Option<String> {
        self.first_related(action, "route")?.owned_text("url")
    }

    pub(super) fn actions(&self, item: &Resource) -> Vec<Action> {
        let mut actions: Vec<Action> = self
            .related(item, "defaultAction")
            .chain(self.related(item, "userActions"))
            .filter_map(|action| {
                let elements = &action.attributes["elements"];
                Some(Action {
                    label: elements["label"]["label"].as_str()?.to_string(),
                    context: action.text("context").unwrap_or_default().to_string(),
                    progress: elements["progress"]["percentComplete"].as_f64().map(|fraction| fraction as f32),
                    live: elements["progress"]["variant"].as_str() == Some("secondary"),
                    route: self.action_route(action),
                })
            })
            .collect();
        actions.dedup();
        actions
    }
}
