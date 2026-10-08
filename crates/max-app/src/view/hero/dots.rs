use std::f32::consts::{FRAC_PI_2, TAU};

use eframe::egui::{self, Color32, Id, Painter, Pos2, Rect, Sense, Shape, Stroke, Vec2};

use super::HeroScene;
use crate::{
    app::App,
    intent::Intent,
    metrics::hero::{DOTS_ABOVE_FIRST_RAIL, DOT_RADIUS, DOT_SPACING, PROGRESS_RING_RADIUS, PROGRESS_RING_WIDTH},
    model::Selection,
    paint::points_to_when_hovered,
    theme::{self, TEXT},
};

const RING_SEGMENTS: usize = 72;
const RESTING_DOT_OPACITY: f32 = 0.45;
const HOVERED_DOT_OPACITY: f32 = 0.75;

impl App {
    pub(super) fn draw_hero_dots(&self, ui: &mut egui::Ui, scene: &HeroScene, intent: &mut Intent) {
        let count = scene.row.tiles.len();
        let first_center = scene.bounds.center().x - (count as f32 - 1.0) * DOT_SPACING / 2.0;
        for dot in 0..count {
            let center = Pos2::new(first_center + dot as f32 * DOT_SPACING, scene.flow.bottom() - DOTS_ABOVE_FIRST_RAIL);
            let target = Rect::from_center_size(center, Vec2::splat(DOT_SPACING));
            let response = ui.interact(target, Id::new(("hero-dot", self.navigation.generation, dot)), Sense::click());
            points_to_when_hovered(ui, &response);
            if response.clicked() {
                intent.select = Some(Selection::new(0, dot));
            }
            let showing = dot == scene.index;
            match self.hero.progress.filter(|_| showing) {
                Some(progress) => paint_progress_ring(&scene.painter, center, progress),
                None => {
                    let color = match (showing, response.hovered()) {
                        (true, _) => TEXT,
                        (false, true) => theme::white(HOVERED_DOT_OPACITY),
                        (false, false) => theme::white(RESTING_DOT_OPACITY),
                    };
                    scene.painter.circle_filled(center, DOT_RADIUS, color);
                }
            }
        }
    }
}

/// The ring and its filled part are lines through the same points, so that one lies exactly
/// over the other.
fn paint_progress_ring(painter: &Painter, center: Pos2, progress: f32) {
    let radius = PROGRESS_RING_RADIUS - PROGRESS_RING_WIDTH / 2.0;
    let point_at = |turn: f32| center + radius * Vec2::angled(TAU * turn - FRAC_PI_2);
    let ring = || (0..RING_SEGMENTS).map(|segment| point_at(segment as f32 / RING_SEGMENTS as f32)).collect::<Vec<_>>();
    let stroke = |color: Color32| Stroke::new(PROGRESS_RING_WIDTH, color);
    painter.add(Shape::closed_line(ring(), stroke(theme::white(RESTING_DOT_OPACITY))));
    if progress >= 1.0 {
        painter.add(Shape::closed_line(ring(), stroke(TEXT)));
    } else if progress > 0.0 {
        let whole_segments = (progress * RING_SEGMENTS as f32).floor() as usize;
        let mut filled: Vec<Pos2> = (0..=whole_segments).map(|segment| point_at(segment as f32 / RING_SEGMENTS as f32)).collect();
        filled.push(point_at(progress));
        painter.add(Shape::line(filled, stroke(TEXT)));
    }
}
