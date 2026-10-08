//! The player's icons, drawn to the proportions of the service's own.

use eframe::egui::{Align2, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use crate::theme::{bold, TEXT};

const LINE: f32 = 1.8;

fn stroke() -> Stroke {
    Stroke::new(LINE, TEXT)
}

fn arc(painter: &Painter, center: Pos2, radius: f32, from_degrees: f32, to_degrees: f32, mirrored: bool, width: f32) {
    const SEGMENTS: usize = 40;
    let across = if mirrored { -1.0 } else { 1.0 };
    let points = (0..=SEGMENTS)
        .map(|step| {
            let angle = (from_degrees + (to_degrees - from_degrees) * step as f32 / SEGMENTS as f32).to_radians();
            center + radius * Vec2::new(across * angle.cos(), angle.sin())
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(width, TEXT)));
}

pub(super) fn chevron_left(painter: &Painter, center: Pos2, arm: f32) {
    let tip = center - Vec2::new(arm / 2.0, 0.0);
    painter.line_segment([tip, tip + Vec2::new(arm, -arm)], stroke());
    painter.line_segment([tip, tip + Vec2::new(arm, arm)], stroke());
}

pub(super) fn pause(painter: &Painter, center: Pos2) {
    const BAR: Vec2 = Vec2::new(2.2, 24.0);
    const BETWEEN_BARS: f32 = 9.2;
    for side in [-1.0, 1.0] {
        painter.rect_filled(Rect::from_center_size(center + Vec2::new(side * BETWEEN_BARS / 2.0, 0.0), BAR), 1.0, TEXT);
    }
}

/// A ring open at the upper side it turns towards, where a square-cornered arrow sits, with
/// "10" in the middle. Skipping forward is the mirror image of skipping back.
pub(super) fn skip(painter: &Painter, center: Pos2, forward: bool) {
    const RADIUS: f32 = 12.6;
    const RING_WIDTH: f32 = 2.2;
    const RING_FROM_DEGREES: f32 = -145.0;
    const RING_TO_DEGREES: f32 = 164.0;
    const ARROW_TOP: Vec2 = Vec2::new(-12.6, -11.6);
    const ARROW_CORNER: Vec2 = Vec2::new(-12.6, -4.95);
    const ARROW_END: Vec2 = Vec2::new(-5.9, -4.95);
    const NUMBER_TEXT: f32 = 9.5;
    const NUMBER_OFFSET: Vec2 = Vec2::new(0.2, 0.6);

    arc(painter, center, RADIUS, RING_FROM_DEGREES, RING_TO_DEGREES, forward, RING_WIDTH);
    let across = if forward { -1.0 } else { 1.0 };
    let place = |offset: Vec2| center + Vec2::new(across * offset.x, offset.y);
    let arrow = Stroke::new(RING_WIDTH, TEXT);
    painter.line_segment([place(ARROW_TOP), place(ARROW_CORNER + Vec2::new(0.0, RING_WIDTH / 2.0))], arrow);
    painter.line_segment([place(ARROW_CORNER), place(ARROW_END)], arrow);
    painter.text(center + NUMBER_OFFSET, Align2::CENTER_CENTER, "10", bold(NUMBER_TEXT), TEXT);
}

pub(super) fn full_screen(painter: &Painter, center: Pos2) {
    const CORNER_FROM_CENTER: f32 = 8.0;
    const ARM: f32 = 5.0;
    for (across, down) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let corner = center + Vec2::new(across, down) * CORNER_FROM_CENTER;
        painter.line_segment([corner + Vec2::new(across * LINE / 2.0, 0.0), corner - Vec2::new(across * ARM, 0.0)], stroke());
        painter.line_segment([corner, corner - Vec2::new(0.0, down * ARM)], stroke());
    }
}

/// A loudspeaker in outline with more waves the louder it is, or a cross when silent.
pub(super) fn volume(painter: &Painter, center: Pos2, heard: f32) {
    const SPEAKER: [Vec2; 6] =
        [Vec2::new(-11.7, -3.9), Vec2::new(-6.0, -3.9), Vec2::new(-1.3, -7.8), Vec2::new(-1.3, 7.8), Vec2::new(-6.0, 3.9), Vec2::new(-11.7, 3.9)];
    const WAVES_FROM: Vec2 = Vec2::new(-2.0, 0.0);
    const WAVES: [(f32, f32); 3] = [(4.7, 42.0), (8.7, 44.0), (12.7, 45.0)];
    const CROSS_CENTER: Vec2 = Vec2::new(7.2, 0.0);
    const CROSS_ARM: f32 = 4.2;
    const THIN: f32 = 2.0;

    let outline = SPEAKER.iter().map(|offset| center + *offset).collect();
    painter.add(Shape::closed_line(outline, Stroke::new(THIN, TEXT)));
    if heard <= 0.0 {
        let middle = center + CROSS_CENTER;
        painter.line_segment([middle - Vec2::splat(CROSS_ARM), middle + Vec2::splat(CROSS_ARM)], Stroke::new(THIN, TEXT));
        painter.line_segment([middle + Vec2::new(-CROSS_ARM, CROSS_ARM), middle + Vec2::new(CROSS_ARM, -CROSS_ARM)], Stroke::new(THIN, TEXT));
        return;
    }
    let waves = (heard * WAVES.len() as f32).ceil() as usize;
    for (radius, half_angle) in WAVES.into_iter().take(waves) {
        arc(painter, center + WAVES_FROM, radius, -half_angle, half_angle, false, THIN);
    }
}
