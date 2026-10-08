use std::{cell::Cell, collections::HashMap, ffi::c_void, ptr::NonNull, sync::Arc, time::Instant};

use max_api::{
    client::Client,
    cms::NextVideo,
    playback::{Section, TitlePlayback, Watching, MASTER_PLAYLIST},
};
use max_media::video::{ProtectedTitle, ProtectedVideo};

mod offers;
mod progress;
mod sound;
mod transport;

pub use sound::Sound;

use crate::model::EpisodesPanel;

const EPISODE: &str = "episode";

pub struct PlayingEpisode {
    pub show_id: String,
    pub season_number: Option<u32>,
}

pub struct Heading {
    pub title: String,
    pub subtitle: Option<String>,
}

pub struct NativePlayer {
    fallback_title: String,
    muted: Cell<bool>,
    volume: Cell<f32>,
    video: ProtectedVideo,
    client: Arc<Client>,
    watching: Watching,
    sections: Vec<Section>,
    watching_credits: Cell<bool>,
    pub next: Option<NextVideo>,
    progress_reported: Instant,
    progress_logged: Instant,
    pub controls_active: Cell<Instant>,
    pub episodes: Option<EpisodesPanel>,
}

impl NativePlayer {
    pub fn open(
        client: Arc<Client>,
        title: String,
        playback: TitlePlayback,
        window_view: NonNull<c_void>,
        sound: Sound,
    ) -> Result<Self, max_media::Error> {
        let (licence_url, key_client) = (playback.fairplay.licence_url, client.clone());
        let protected = ProtectedTitle {
            playlists: playback.playlists.into_iter().map(|playlist| (playlist.name, playlist.text)).collect::<HashMap<_, _>>(),
            master_playlist: MASTER_PLAYLIST.to_string(),
            certificate: playback.fairplay.certificate,
            fetch_key: Arc::new(move |key_id: &str, key_request: &[u8]| key_client.fairplay_key(&licence_url, key_id, key_request)),
        };
        let video = ProtectedVideo::open(protected)?;
        // SAFETY: the pointer is the window's own view, taken from the toolkit this frame, on
        // the main thread.
        unsafe { video.show_in(window_view.as_ptr()) };
        video.set_muted(sound.muted);
        if let Some(start) = playback.start_seconds.filter(|start| *start > 0.0) {
            video.seek(start);
        }
        video.set_playing(true);
        video.set_volume(sound.volume);
        Ok(Self {
            fallback_title: title,
            muted: Cell::new(sound.muted),
            volume: Cell::new(sound.volume),
            video,
            client,
            watching: playback.watching,
            sections: playback.sections,
            watching_credits: Cell::new(false),
            next: None,
            progress_reported: Instant::now(),
            progress_logged: Instant::now(),
            controls_active: Cell::new(Instant::now()),
            episodes: None,
        })
    }

    pub fn video_id(&self) -> &str {
        &self.watching.video.video_id
    }

    pub fn episode(&self) -> Option<PlayingEpisode> {
        let video = &self.watching.video;
        let show_id = video.show_id.clone().filter(|_| video.video_type == EPISODE)?;
        Some(PlayingEpisode { show_id, season_number: video.season_number })
    }

    /// An episode is headed by its series, with its place in it and its own name beneath.
    pub fn heading(&self) -> Heading {
        let video = &self.watching.video;
        let own_name = if video.name.is_empty() { self.fallback_title.clone() } else { video.name.clone() };
        match (&video.show_name, video.season_number.zip(video.episode_number)) {
            (Some(show), Some((season, episode))) if video.video_type == EPISODE => {
                Heading { title: show.clone(), subtitle: Some(format!("S{season} E{episode}:  {own_name}")) }
            }
            _ => Heading { title: own_name, subtitle: None },
        }
    }

    pub fn show_controls(&self) {
        self.controls_active.set(Instant::now());
    }

    pub fn failure(&self) -> Option<String> {
        self.video.failure()
    }

    pub fn finished(&self) -> bool {
        self.video.finished()
    }
}
