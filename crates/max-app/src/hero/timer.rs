use std::time::{Duration, Instant};

use eframe::egui;

use super::PreviewStage;
use crate::{
    app::App,
    timing::{HERO_DWELL_WITHOUT_PREVIEW_SECONDS, PREVIEW_FADE},
};

const LONGEST_TICK_SECONDS: f32 = 0.25;
const TICK: Duration = Duration::from_millis(33);

struct Timed {
    progress: f32,
    done: bool,
}

impl App {
    pub(crate) fn run_hero_timer(&mut self, ctx: &egui::Context) {
        let title = self.hero_title();
        let since_last_tick = std::mem::replace(&mut self.hero.last_tick, Instant::now()).elapsed().as_secs_f32().min(LONGEST_TICK_SECONDS);
        if self.hero.timed_title != title {
            self.hero.timed_title = title;
            self.hero.seconds_in_view = 0.0;
        }
        self.hero.progress = None;
        if self.page.hero_len().is_none_or(|len| len < 2) {
            return;
        }
        let watchable = self.hero_can_be_watched();
        let has_preview = self.service.is_live() && self.hero_tile().is_some_and(|tile| tile.detail.preview_edit_id.is_some());
        let stage = self.hero.preview.as_ref().filter(|preview| preview.title == title).map(|preview| &preview.stage);
        let timed = match stage {
            Some(PreviewStage::Playing { clip, first_frame_at: Some(_), .. }) => Timed { progress: clip.progress().unwrap_or(0.0), done: false },
            Some(PreviewStage::Ended { since, .. }) => Timed { progress: 1.0, done: since.elapsed() >= PREVIEW_FADE },
            Some(PreviewStage::Finding | PreviewStage::Playing { .. }) => Timed { progress: 0.0, done: false },
            None if has_preview => Timed { progress: 0.0, done: false },
            Some(PreviewStage::Unavailable) | None => {
                if watchable {
                    self.hero.seconds_in_view += since_last_tick;
                    ctx.request_repaint_after(TICK);
                }
                let fraction = self.hero.seconds_in_view / HERO_DWELL_WITHOUT_PREVIEW_SECONDS;
                Timed { progress: fraction, done: fraction >= 1.0 }
            }
        };
        self.hero.progress = Some(timed.progress.clamp(0.0, 1.0));
        if timed.done && watchable {
            self.step_hero(true);
        }
    }
}
