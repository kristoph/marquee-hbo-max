use serde_json::Value;

use super::Client;
use crate::{
    cms::{Document, NextVideo},
    Error,
};

impl Client {
    /// What follows a video in its series' own order, if anything does.
    pub fn next_video(&self, video_id: &str) -> Result<Option<NextVideo>, Error> {
        let url =
            format!("{}/cms/recommendations/nextVideos?algorithm=naturalOrder&videoId={video_id}&include=default", self.session().content_origin());
        let mut answer: Value = serde_json::from_str(&self.get(&url)?)?;
        // The answer lists videos where a document has one resource at its root.
        let Some(first) = answer["data"].as_array_mut().filter(|videos| !videos.is_empty()).map(|videos| videos.swap_remove(0)) else {
            return Ok(None);
        };
        answer["data"] = first;
        Ok(Document::from_value(answer)?.next_video())
    }
}
