use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use super::sections::{Section, SectionKind};
use crate::{
    client::{Body, Client},
    cms::PlayableVideo,
    Error,
};

const MAIN_VIDEO: &str = "main";

/// What the service is told, along with the position, while a title plays: this is what keeps
/// Continue Watching and resuming in step with what was watched.
#[derive(Debug, Clone, PartialEq)]
pub struct Watching {
    pub video: PlayableVideo,
    pub playback_session_id: String,
    pub application_session_id: String,
    pub runtime_seconds: f64,
    pub credits_start_seconds: Option<f64>,
}

pub(super) fn main_video(playback_answer: &Value) -> Option<&Value> {
    playback_answer["videos"].as_array()?.iter().find(|video| video["type"] == MAIN_VIDEO)
}

impl Watching {
    pub(super) fn of(video: PlayableVideo, session_ids: &SessionIds, main_video: &Value, sections: &[Section]) -> Option<Self> {
        let credits = sections.iter().rfind(|section| section.kind == SectionKind::Credits);
        Some(Self {
            video,
            playback_session_id: session_ids.playback.clone(),
            application_session_id: session_ids.application.clone(),
            runtime_seconds: main_video["duration"].as_f64()?,
            credits_start_seconds: credits.map(|credits| credits.start_seconds),
        })
    }

    fn marker(&self, position_seconds: f64, now_milliseconds: u128) -> Value {
        json!({
            "editId": self.video.edit_id,
            "positionSec": position_seconds,
            "runtimeSec": self.runtime_seconds,
            "videoType": self.video.video_type,
            "playbackSessionId": self.playback_session_id,
            "playableStatus": "VOD",
            "creationTimeEpochMs": now_milliseconds as u64,
            "programId": self.video.video_id,
            "showId": self.video.show_id,
            "creditsStartTimeSec": self.credits_start_seconds,
            "mainContentDurationSec": self.runtime_seconds,
            "appSessionId": self.application_session_id,
        })
    }
}

pub struct SessionIds {
    pub playback: String,
    pub application: String,
}

impl Client {
    pub fn report_progress(&self, watching: &Watching, position_seconds: f64) -> Result<(), Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let url = format!("{}/markers/any/markers/v1/markers", self.session().progress_origin());
        self.send("POST", &url, Body::Json(&watching.marker(position_seconds, now).to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
