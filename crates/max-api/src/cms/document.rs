use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum Related {
    One(ResourceKey),
    Many(Vec<ResourceKey>),
    #[default]
    Nothing,
}

#[derive(Debug, Default, Deserialize)]
struct Relationship {
    #[serde(default)]
    data: Related,
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    #[serde(default)]
    pub attributes: Value,
    #[serde(default)]
    relationships: HashMap<String, Relationship>,
}

impl Resource {
    pub fn related_keys(&self, relationship: &str) -> &[ResourceKey] {
        match self.relationships.get(relationship).map(|relationship| &relationship.data) {
            Some(Related::One(key)) => std::slice::from_ref(key),
            Some(Related::Many(keys)) => keys,
            _ => &[],
        }
    }

    pub fn text(&self, attribute: &str) -> Option<&str> {
        self.attributes.get(attribute)?.as_str()
    }

    pub fn owned_text(&self, attribute: &str) -> Option<String> {
        self.text(attribute).map(str::to_string)
    }
}

#[derive(Debug, Deserialize)]
struct RawDocument {
    data: Resource,
    #[serde(default)]
    included: Vec<Resource>,
}

#[derive(Debug)]
pub struct Document {
    pub root: Resource,
    included: HashMap<ResourceKey, Resource>,
}

impl Document {
    /// Also accepts a response as saved by the web capture tool, which wraps it under `body`.
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        let mut value: Value = serde_json::from_str(json)?;
        if value.get("capture").is_some() {
            value = value["body"].take();
        }
        Self::from_value(value)
    }

    pub fn from_value(value: Value) -> Result<Self, serde_json::Error> {
        let raw: RawDocument = serde_json::from_value(value)?;
        let included =
            raw.included.into_iter().map(|resource| (ResourceKey { kind: resource.kind.clone(), id: resource.id.clone() }, resource)).collect();
        Ok(Self { root: raw.data, included })
    }

    pub fn get(&self, key: &ResourceKey) -> Option<&Resource> {
        self.included.get(key)
    }

    pub(super) fn included(&self) -> impl Iterator<Item = &Resource> {
        self.included.values()
    }

    pub(super) fn related<'document>(
        &'document self,
        from: &'document Resource,
        relationship: &str,
    ) -> impl Iterator<Item = &'document Resource> + 'document {
        from.related_keys(relationship).iter().filter_map(move |key| self.get(key))
    }

    pub(super) fn first_related<'document>(&'document self, from: &'document Resource, relationship: &str) -> Option<&'document Resource> {
        self.related(from, relationship).next()
    }

    pub(super) fn labels(&self, from: &Resource, relationship: &str) -> Vec<String> {
        self.related(from, relationship)
            .filter_map(|labelled| ["label", "name", "code"].iter().find_map(|attribute| labelled.text(attribute)))
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .map(str::to_string)
            .collect()
    }
}

pub(super) fn text_at(value: &Value, path: &[&str]) -> Option<String> {
    path.iter().try_fold(value, |value, key| value.get(key)).and_then(Value::as_str).map(str::to_string)
}
