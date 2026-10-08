use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::{
    client::{Body, Client},
    cms::PlayableVideo,
    Error,
};

const MAIN_VIDEO: &str = "main";
const END_CREDITS: &str = "end-credits";

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
    /// Times in the playback answer count from the start of the unprotected opening; the
    /// service wants them counted from the start of the title itself.
    pub(super) fn of(video: PlayableVideo, session_ids: &SessionIds, playback_answer: &Value) -> Option<Self> {
        let main = main_video(playback_answer)?;
        let start = main["start"].as_f64().unwrap_or(0.0);
        let credits = main["annotations"].as_array().into_iter().flatten().rfind(|annotation| annotation["type"] == END_CREDITS);
        Some(Self {
            video,
            playback_session_id: session_ids.playback.clone(),
            application_session_id: session_ids.application.clone(),
            runtime_seconds: main["duration"].as_f64()?,
            credits_start_seconds: credits.and_then(|credits| credits["start"].as_f64()).map(|credits_start| credits_start - start),
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
mod tests {
    use super::*;

    fn watching() -> Watching {
        let video = PlayableVideo {
            video_id: "v1".to_string(),
            edit_id: "e1".to_string(),
            show_id: Some("s1".to_string()),
            video_type: "movie".to_string(),
            name: "A Film".to_string(),
            show_name: None,
            season_number: None,
            episode_number: None,
            resume_seconds: None,
        };
        let answer = json!({"videos": [
            {"type": "promo", "start": 0, "duration": 14.0},
            {"type": "main", "start": 14.0, "duration": 7760.5,
             "annotations": [{"type": "end-credits", "start": 7156.0}, {"type": "end-credits", "start": 7291.5}]}
        ]});
        Watching::of(video, &SessionIds { playback: "p".to_string(), application: "a".to_string() }, &answer).unwrap()
    }

    #[test]
    fn takes_the_runtime_and_credits_of_the_main_video() {
        let watching = watching();
        assert_eq!((watching.runtime_seconds, watching.credits_start_seconds), (7760.5, Some(7277.5)));
    }

    #[test]
    fn a_marker_says_where_in_which_title_and_when() {
        assert_eq!(
            watching().marker(60.5, 1_791_475_368_171),
            json!({
                "editId": "e1", "positionSec": 60.5, "runtimeSec": 7760.5, "videoType": "movie", "playbackSessionId": "p",
                "playableStatus": "VOD", "creationTimeEpochMs": 1_791_475_368_171_u64, "programId": "v1", "showId": "s1",
                "creditsStartTimeSec": 7277.5, "mainContentDurationSec": 7760.5, "appSessionId": "a"
            })
        );
    }
}
