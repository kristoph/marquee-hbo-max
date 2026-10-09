use super::document::Document;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayableVideo {
    pub video_id: String,
    pub edit_id: String,
    pub show_id: Option<String>,
    pub video_type: String,
    pub name: String,
    pub show_name: Option<String>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub resume_seconds: Option<f64>,
}

impl Document {
    pub fn playable_video(&self, video_id: &str) -> Option<PlayableVideo> {
        let video = self.included().find(|resource| resource.kind == "video" && resource.id == video_id)?;
        let history = &video.attributes["viewingHistory"];
        let unfinished = history["completed"].as_bool() != Some(true);
        let number = |attribute: &str| video.attributes[attribute].as_u64().map(|number| number as u32);
        Some(PlayableVideo {
            video_id: video_id.to_string(),
            edit_id: video.related_keys("edit").first()?.id.clone(),
            show_id: video.related_keys("show").first().map(|show| show.id.clone()),
            video_type: video.text("videoType").unwrap_or_default().to_lowercase(),
            name: video.text("name").unwrap_or_default().to_string(),
            show_name: self.first_related(video, "show").and_then(|show| show.owned_text("name")),
            season_number: number("seasonNumber"),
            episode_number: number("episodeNumber"),
            resume_seconds: history["position"].as_f64().filter(|_| unfinished).map(|milliseconds| milliseconds / 1000.0),
        })
    }
}

/// The video that follows another, as much of it as offering to play it takes.
#[derive(Debug, Clone, PartialEq)]
pub struct NextVideo {
    pub route: String,
    pub name: String,
    pub season_and_episode: Option<(u32, u32)>,
}

impl Document {
    /// Reads a document whose root is the video itself.
    pub fn next_video(&self) -> Option<NextVideo> {
        let video = Some(&self.root).filter(|root| root.kind == "video")?;
        let number = |attribute: &str| video.attributes[attribute].as_u64().map(|number| number as u32);
        Some(NextVideo {
            route: format!("/video/watch/{}", video.id),
            name: video.text("name").unwrap_or_default().to_string(),
            season_and_episode: number("seasonNumber").zip(number("episodeNumber")),
        })
    }
}

#[cfg(test)]
mod tests;
