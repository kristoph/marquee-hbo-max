use std::time::Instant;

use max_api::cms::Filter;

use super::{page::SearchSource, rail::ScreenRow};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchQuery {
    pub text: String,
    pub topic_parameters: Option<String>,
}

impl SearchQuery {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.topic_parameters.is_none()
    }
}

pub struct Search {
    pub results_id: String,
    pub text: String,
    pub topic: Option<usize>,
    pub shown: SearchQuery,
    pub edited: Option<Instant>,
    pub topics: Vec<Filter>,
    pub starting_topics: Vec<Filter>,
    pub rows_before_searching: Option<Vec<ScreenRow>>,
    pub focus_field: bool,
}

impl Search {
    pub fn new(source: SearchSource) -> Self {
        Self {
            results_id: source.results_id,
            text: String::new(),
            topic: None,
            shown: SearchQuery::default(),
            edited: None,
            starting_topics: source.topics.clone(),
            topics: source.topics,
            rows_before_searching: None,
            focus_field: true,
        }
    }

    pub fn source(&self) -> SearchSource {
        SearchSource { results_id: self.results_id.clone(), topics: self.starting_topics.clone() }
    }

    pub fn wanted(&self) -> SearchQuery {
        let topic = self.topic.and_then(|index| self.topics.get(index));
        SearchQuery { text: self.text.trim().to_string(), topic_parameters: topic.map(|topic| topic.parameter.clone()) }
    }
}
