mod script;
mod steps;

use std::time::{Duration, Instant};

use eframe::egui::{self, Key};
pub use script::Step;

use crate::{app::App, intent::Intent};

const START_KEY: Key = Key::D;
const LEAD_BEFORE_UNATTENDED_START: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(30);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WalkthroughMode {
    StartOnKeyPress,
    StartUnattended,
    DryRun,
}

pub struct Walkthrough {
    mode: WalkthroughMode,
    first_page_shown_at: Option<Instant>,
    started: Option<Instant>,
    next_step: usize,
}

impl Walkthrough {
    pub fn new(mode: WalkthroughMode) -> Self {
        Self { mode, first_page_shown_at: None, started: None, next_step: 0 }
    }

    fn should_start(&mut self, ctx: &egui::Context) -> bool {
        let key_pressed = !ctx.egui_wants_keyboard_input() && ctx.input(|input| input.key_pressed(START_KEY));
        let lead = match self.mode {
            WalkthroughMode::StartOnKeyPress => return key_pressed,
            WalkthroughMode::StartUnattended => LEAD_BEFORE_UNATTENDED_START,
            WalkthroughMode::DryRun => Duration::ZERO,
        };
        ctx.request_repaint_after(POLL);
        key_pressed || self.first_page_shown_at.get_or_insert_with(Instant::now).elapsed() >= lead
    }
}

impl App {
    pub(crate) fn run_walkthrough(&mut self, ctx: &egui::Context, intent: &mut Intent) {
        let busy = self.fade.is_some() || self.navigation.loading.is_some() || self.page.rows.is_empty();
        let Some(walkthrough) = &mut self.developer.walkthrough else { return };
        let Some(started) = walkthrough.started else {
            if !busy && walkthrough.should_start(ctx) {
                walkthrough.started = Some(Instant::now());
                log::info!("WALKTHROUGH started");
                ctx.request_repaint();
            }
            return;
        };
        let Some(scheduled) = script::SCRIPT.get(walkthrough.next_step) else {
            log::info!("WALKTHROUGH finished");
            self.developer.walkthrough = None;
            return;
        };
        ctx.request_repaint_after(POLL);
        let (elapsed, step_number, dry_run) = (started.elapsed().as_secs_f32(), walkthrough.next_step, walkthrough.mode == WalkthroughMode::DryRun);
        if elapsed < scheduled.at_seconds {
            return;
        }
        let attempt = steps::Attempt { seconds_overdue: elapsed - scheduled.at_seconds, dry_run, busy };
        if self.perform(scheduled.step, attempt, intent) {
            log::info!("WALKTHROUGH {elapsed:5.1}s step {step_number}");
            if let Some(walkthrough) = &mut self.developer.walkthrough {
                walkthrough.next_step += 1;
            }
        }
    }
}
