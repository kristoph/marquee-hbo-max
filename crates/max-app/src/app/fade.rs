use std::time::Instant;

use eframe::egui::{self, Id, LayerId, Order, Shape, Stroke, Vec2};

use super::App;
use crate::player::PendingPlayback;
use crate::{
    theme,
    timing::{FADE_IN, FADE_OUT, SPINNER_DELAY, WAIT_FOR_FIRST_SCREEN_ARTWORK},
    transition::{Arrival, Fade, FadePhase},
};

const SPINNER_RADIUS: f32 = 22.0;
const SPINNER_WIDTH: f32 = 4.0;
const SPINNER_SEGMENTS: usize = 36;
const SPINNER_FRACTION_OF_A_RING: f32 = 0.75;

impl App {
    /// A page that was waiting its turn is not lost when something else arrives on top of it.
    pub(crate) fn arrive(&mut self, arrival: Arrival) {
        if let Some(Arrival::Page { route, remember, page }) = self.fade.as_mut().and_then(|fade| fade.arrival.take()) {
            self.show_page(route, remember, page);
        }
        match &mut self.fade {
            Some(fade) if fade.phase != FadePhase::In => fade.deliver(arrival),
            _ => self.fade.insert(Fade::starting(FadePhase::Black)).deliver(arrival),
        }
    }

    pub(crate) fn fade_in(&mut self) {
        self.fade = Some(Fade::starting(FadePhase::In));
    }

    pub(crate) fn run_fade(&mut self, ctx: &egui::Context) {
        let page_is_covered = self.something_covers_the_page();
        let Some(fade) = &mut self.fade else { return };
        if !page_is_covered {
            ctx.request_repaint();
        }
        let elapsed = fade.since.elapsed();
        match fade.phase {
            FadePhase::Out if elapsed >= FADE_OUT => {
                fade.phase = FadePhase::Black;
                fade.since = Instant::now();
            }
            FadePhase::In if elapsed >= FADE_IN => self.fade = None,
            FadePhase::Black if is_waiting_for_artwork(fade) => {}
            FadePhase::Black => match fade.arrival.take() {
                Some(Arrival::Page { route, remember, page }) => {
                    self.show_page(route, remember, page);
                    self.fade_in();
                }
                Some(Arrival::Play { route, title }) => self.playback.to_open = Some(PendingPlayback { route, title }),
                Some(Arrival::Failed(reason)) => {
                    self.say(reason);
                    self.fade_in();
                }
                None => {}
            },
            _ => {}
        }
    }

    pub(crate) fn draw_fade(&self, ctx: &egui::Context) {
        let Some(fade) = &self.fade else { return };
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("fade")));
        let window = ctx.content_rect();
        painter.rect_filled(window, 0.0, egui::Color32::from_black_alpha((fade.darkness() * 255.0).round() as u8));
        if fade.phase == FadePhase::Black && fade.since.elapsed() >= SPINNER_DELAY && !self.something_covers_the_page() {
            let turn = ctx.input(|input| input.time) as f32 * std::f32::consts::TAU;
            let points = (0..=SPINNER_SEGMENTS).map(|segment| {
                let angle = turn + segment as f32 / SPINNER_SEGMENTS as f32 * std::f32::consts::TAU * SPINNER_FRACTION_OF_A_RING;
                window.center() + Vec2::angled(angle) * SPINNER_RADIUS
            });
            painter.add(Shape::line(points.collect(), Stroke::new(SPINNER_WIDTH, theme::TEXT)));
        }
    }
}

fn is_waiting_for_artwork(fade: &Fade) -> bool {
    let page_lacks_artwork = matches!(&fade.arrival, Some(Arrival::Page { page, .. }) if !page.first_screen_ready());
    page_lacks_artwork && fade.arrived.is_some_and(|arrived| arrived.elapsed() < WAIT_FOR_FIRST_SCREEN_ARTWORK)
}
