mod preview;
mod stepping;
mod swipe;
mod timer;

use std::{cell::Cell, time::Instant};

pub use preview::{HeroPreview, PreviewStage};
pub use swipe::Swipes;

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeroTitle {
    pub page_generation: u64,
    pub index: usize,
}

pub struct Slide {
    pub from: usize,
    pub started: Instant,
    pub forward: bool,
}

pub struct HeroState {
    pub index: usize,
    pub slide: Option<Slide>,
    pub shown_title: HeroTitle,
    pub shown_since: Instant,
    pub preview: Option<HeroPreview>,
    pub preview_muted: bool,
    pub timed_title: HeroTitle,
    pub seconds_in_view: f32,
    pub last_tick: Instant,
    pub progress: Option<f32>,
    pub in_view: Cell<bool>,
    pub drag: f32,
    pub swipes: Swipes,
}

impl HeroState {
    pub fn showing(index: usize) -> Self {
        Self {
            index,
            slide: None,
            shown_title: HeroTitle::default(),
            shown_since: Instant::now(),
            preview: None,
            preview_muted: true,
            timed_title: HeroTitle::default(),
            seconds_in_view: 0.0,
            last_tick: Instant::now(),
            progress: None,
            in_view: Cell::new(false),
            drag: 0.0,
            swipes: Swipes::default(),
        }
    }
}

impl App {
    pub(crate) fn hero_title(&self) -> HeroTitle {
        HeroTitle { page_generation: self.navigation.generation, index: self.hero.index }
    }

    pub(crate) fn hero_can_be_watched(&self) -> bool {
        let covered = self.playback.is_open() || self.fade.is_some() || self.browse_menu_open || self.tile_menu.is_some();
        self.hero.in_view.get() && !covered
    }
}
