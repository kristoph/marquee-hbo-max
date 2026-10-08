use std::time::{Duration, Instant};

use eframe::egui;

use super::Slide;
use crate::{app::App, intent::Intent, metrics::hero::DRAG_DISTANCE_FOR_A_STEP, model::ScreenTile};

const SWIPE_POLL: Duration = Duration::from_millis(50);

impl App {
    pub(crate) fn hero_tile(&self) -> Option<&ScreenTile> {
        self.page.hero()?.tiles.get(self.hero.index)
    }

    pub(crate) fn set_hero(&mut self, index: usize) {
        let Some(len) = self.page.hero_len() else { return };
        let index = index.min(len - 1);
        if index != self.hero.index {
            self.hero.slide = Some(Slide { from: self.hero.index, started: Instant::now(), forward: index > self.hero.index });
            self.hero.index = index;
        }
        if self.page.selected.row == 0 {
            self.page.selected.column = index;
        }
    }

    pub(crate) fn step_hero(&mut self, forward: bool) {
        let Some(len) = self.page.hero_len() else { return };
        self.set_hero(if forward { (self.hero.index + 1) % len } else { (self.hero.index + len - 1) % len });
        // Wrapping round from the last title to the first still slides the way the step went.
        if let Some(slide) = &mut self.hero.slide {
            slide.forward = forward;
        }
    }

    pub(crate) fn swipe_hero(&mut self, ctx: &egui::Context, intent: &Intent) {
        self.hero.drag += intent.hero_drag;
        if intent.hero_drag_ended {
            let travelled = std::mem::take(&mut self.hero.drag);
            if travelled.abs() >= DRAG_DISTANCE_FOR_A_STEP {
                self.step_hero(travelled < 0.0);
            }
        }
        let now = Instant::now();
        for sideways in &intent.hero_sideways_scrolls {
            if let Some(forward) = self.hero.swipes.feed(*sideways, now) {
                self.step_hero(forward);
            }
        }
        if self.hero.swipes.is_active(now) {
            ctx.request_repaint_after(SWIPE_POLL);
        }
    }
}
