use std::{
    cell::Cell,
    collections::HashMap,
    ffi::c_void,
    ptr::NonNull,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use max_api::{
    client::Client,
    cms::NextVideo,
    playback::{Section, TitlePlayback, Watching, MASTER_PLAYLIST},
};
use max_media::video::{ProtectedTitle, ProtectedVideo};

use super::prompts::{Moment, Prompt};
use crate::model::EpisodesPanel;

const SKIP_SECONDS: f64 = 10.0;
const REPORT_PROGRESS_EVERY: Duration = Duration::from_secs(20);
const LOG_PROGRESS_EVERY: Duration = Duration::from_secs(5);

const EPISODE: &str = "episode";

pub struct PlayingEpisode {
    pub show_id: String,
    pub season_number: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound {
    pub muted: bool,
    pub volume: f32,
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

    pub fn is_muted(&self) -> bool {
        self.muted.get()
    }

    pub fn volume(&self) -> f32 {
        self.volume.get()
    }

    /// Moving the volume off silence unmutes, as a person would expect of a slider.
    pub fn set_volume(&self, volume: f32) {
        self.volume.set(volume);
        self.video.set_volume(volume);
        self.muted.set(volume <= 0.0);
        self.video.set_muted(self.muted.get());
    }

    pub fn sound(&self) -> Sound {
        Sound { muted: self.muted.get(), volume: self.volume.get() }
    }

    pub fn toggle_muted(&self) {
        self.muted.set(!self.muted.get());
        self.video.set_muted(self.muted.get());
    }

    pub fn show_controls(&self) {
        self.controls_active.set(Instant::now());
    }

    pub fn is_playing(&self) -> bool {
        self.video.is_playing()
    }

    pub fn toggle_playing(&self) {
        self.video.set_playing(!self.video.is_playing());
        self.report_progress();
    }

    pub fn position(&self) -> Option<f64> {
        self.video.position()
    }

    pub fn duration(&self) -> Option<f64> {
        self.video.duration().filter(|duration| *duration > 0.0)
    }

    pub fn seek(&self, position_seconds: f64) {
        self.video.seek(position_seconds.clamp(0.0, self.duration().unwrap_or(f64::MAX)));
    }

    pub fn skip(&self, forward: bool) {
        if let Some(position) = self.position() {
            self.seek(position + if forward { SKIP_SECONDS } else { -SKIP_SECONDS });
        }
    }

    pub fn prompt(&self) -> Option<Prompt<'_>> {
        let moment = Moment {
            position_seconds: self.position()?,
            duration_seconds: self.duration()?,
            sections: &self.sections,
            next: self.next.as_ref(),
            watching_credits: self.watching_credits.get(),
        };
        moment.prompt()
    }

    pub fn watch_credits(&self) {
        self.watching_credits.set(true);
    }

    /// The next episode starts by itself when its wait has run out, or when the title ends
    /// before that; someone who chose the credits is left with them.
    pub fn next_when_due(&self) -> Option<&NextVideo> {
        let waited_out = matches!(self.prompt(), Some(Prompt::UpNext { counted_down, .. }) if counted_down >= 1.0);
        self.next.as_ref().filter(|_| waited_out || (self.finished() && !self.watching_credits.get()))
    }

    pub fn failure(&self) -> Option<String> {
        self.video.failure()
    }

    pub fn finished(&self) -> bool {
        self.video.finished()
    }

    pub fn report_progress_when_due(&mut self) {
        if self.is_playing() && self.progress_reported.elapsed() >= REPORT_PROGRESS_EVERY {
            self.progress_reported = Instant::now();
            self.report_progress();
        }
    }

    pub fn report_progress(&self) {
        let Some(position) = self.position() else { return };
        let (client, watching) = (self.client.clone(), self.watching.clone());
        thread::spawn(move || {
            if let Err(error) = client.report_progress(&watching, position) {
                eprintln!("progress could not be reported: {error}");
            }
        });
    }

    pub fn log_progress_when_due(&mut self) {
        if self.progress_logged.elapsed() >= LOG_PROGRESS_EVERY {
            self.progress_logged = Instant::now();
            println!("NATIVE PLAYER {}", self.progress_report());
        }
    }

    fn progress_report(&self) -> String {
        let seconds = |value: Option<f64>| value.map_or_else(|| "?".to_string(), |value| format!("{value:.1}s"));
        format!("at {} of {}, picture ready: {}", seconds(self.position()), seconds(self.duration()), self.video.has_picture())
    }
}
