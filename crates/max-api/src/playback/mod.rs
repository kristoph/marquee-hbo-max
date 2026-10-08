//! Playing a whole title with Apple's player. The service describes the title's media in the
//! DASH format, which that player does not read, so playlists in the HLS format it does read
//! are written over the same media files; the media and its protection are left as they are.

mod index;
mod licence;
mod manifest;
mod playlists;
mod progress;
mod request;
mod sections;

pub use playlists::{Playlist, MASTER_PLAYLIST};
pub(crate) use progress::SessionIds;
pub use progress::Watching;
pub(crate) use request::{playback_request, Purpose};
pub use sections::{Section, SectionKind};
use std::thread;

use serde_json::Value;

use crate::{
    client::{Body, Client},
    cms::{Document, PlayableVideo},
    net, Error,
};

pub struct FairPlay {
    pub certificate: Vec<u8>,
    pub licence_url: String,
}

pub struct TitlePlayback {
    pub playlists: Vec<Playlist>,
    pub fairplay: FairPlay,
    pub watching: Watching,
    pub sections: Vec<Section>,
    pub start_seconds: Option<f64>,
}

impl Client {
    pub fn title_playback(&self, play_route: &str) -> Result<TitlePlayback, Error> {
        let video = self.playable_video(play_route)?;
        let session_ids = SessionIds::new();
        let start_seconds = position_asked_for(play_route).or(video.resume_seconds);
        let device_id = self.session().device_id().ok_or(Error::SignedOut)?;
        let url = format!("{}/any/playback/v1/playbackInfo", self.session().playback_origin());
        let request = playback_request(device_id, &video.edit_id, Purpose::Title, &session_ids).to_string();
        let answer: Value = serde_json::from_str(&self.send("POST", &url, Body::Json(&request))?)?;
        let fairplay = &answer["drm"]["schemes"]["fairplay"];
        let (certificate_url, licence_url) =
            fairplay["certificateUrl"].as_str().zip(fairplay["licenseUrl"].as_str()).ok_or(Error::Lacks("a FairPlay licence for this title"))?;
        let manifest_url = answer["manifest"]["url"].as_str().ok_or(Error::Lacks("a manifest"))?;
        let (playlists, certificate) = thread::scope(|scope| {
            let certificate = scope.spawn(|| net::bytes(certificate_url));
            let playlists = net::text(manifest_url).and_then(|manifest| playlists::write(&manifest::feature(&manifest, manifest_url)?));
            (playlists, certificate.join().expect("the certificate fetch does not panic"))
        });
        Ok(TitlePlayback {
            playlists: playlists?,
            fairplay: FairPlay { certificate: certificate?, licence_url: licence_url.to_string() },
            watching: Watching::of(video, &session_ids, &answer).ok_or(Error::Lacks("a description of the title"))?,
            sections: progress::main_video(&answer).map(sections::sections_of).unwrap_or_default(),
            start_seconds,
        })
    }

    fn playable_video(&self, play_route: &str) -> Result<PlayableVideo, Error> {
        let path = play_route.split(['?', '#']).next().unwrap_or(play_route);
        let video_id = path.rsplit('/').next().unwrap_or_default();
        let document = Document::parse(&self.get(&self.session().page_url(path))?)?;
        document.playable_video(video_id).ok_or(Error::Lacks("anything to play"))
    }
}

/// A menu's "Restart" names the position to start from in its route, as `pos`.
fn position_asked_for(play_route: &str) -> Option<f64> {
    let query = play_route.split_once('?')?.1;
    query.split('&').find_map(|parameter| parameter.strip_prefix("pos=")?.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_starting_position_from_a_route() {
        assert_eq!(position_asked_for("/video/watch/v?pos=0"), Some(0.0));
        assert_eq!(position_asked_for("/video/watch/v?intent=catchup&pos=12.5"), Some(12.5));
        assert_eq!(position_asked_for("/video/watch/v"), None);
    }
}
