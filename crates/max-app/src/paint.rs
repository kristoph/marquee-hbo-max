use std::sync::Arc;

use eframe::egui::{
    self,
    text::{LayoutJob, TextWrapping},
    Color32, CursorIcon, FontId, Galley, Painter, Pos2, Rect, Response, Shape, Stroke, Vec2,
};

#[derive(Clone, Copy)]
pub struct CornerColors {
    pub top_left: Color32,
    pub top_right: Color32,
    pub bottom_left: Color32,
    pub bottom_right: Color32,
}

impl CornerColors {
    pub fn top_to_bottom(top: Color32, bottom: Color32) -> Self {
        Self { top_left: top, top_right: top, bottom_left: bottom, bottom_right: bottom }
    }

    pub fn left_to_right(left: Color32, right: Color32) -> Self {
        Self { top_left: left, top_right: right, bottom_left: left, bottom_right: right }
    }
}

pub fn gradient(painter: &Painter, area: Rect, colors: CornerColors) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(area.left_top(), colors.top_left);
    mesh.colored_vertex(area.right_top(), colors.top_right);
    mesh.colored_vertex(area.left_bottom(), colors.bottom_left);
    mesh.colored_vertex(area.right_bottom(), colors.bottom_right);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(Shape::mesh(mesh));
}

pub struct WrappedText<'text> {
    pub text: &'text str,
    pub font: FontId,
    pub color: Color32,
    pub width: f32,
    pub lines: usize,
}

impl WrappedText<'_> {
    pub fn layout(&self, painter: &Painter) -> Arc<Galley> {
        let mut job = LayoutJob::simple(self.text.to_string(), self.font.clone(), self.color, self.width);
        job.wrap = ellipsis_after(self.lines, self.width);
        painter.layout_job(job)
    }

    pub fn paint(&self, painter: &Painter, position: Pos2) -> f32 {
        let galley = self.layout(painter);
        let height = galley.size().y;
        painter.galley(position, galley, self.color);
        height
    }
}

pub fn ellipsis_after(lines: usize, width: f32) -> TextWrapping {
    TextWrapping { max_width: width, max_rows: lines, break_anywhere: lines == 1, overflow_character: Some('…') }
}

pub fn play_mark(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let points = vec![center + Vec2::new(-radius * 0.75, -radius), center + Vec2::new(radius, 0.0), center + Vec2::new(-radius * 0.75, radius)];
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub fn tick(painter: &Painter, center: Pos2, size: f32, stroke: Stroke) {
    let points = [Vec2::new(-0.89, 0.0), Vec2::new(-0.28, 0.67), Vec2::new(1.0, -0.67)].map(|offset| center + offset * size);
    painter.line_segment([points[0], points[1]], stroke);
    painter.line_segment([points[1], points[2]], stroke);
}

pub fn cross(painter: &Painter, center: Pos2, arm: f32, stroke: Stroke) {
    painter.line_segment([center - Vec2::splat(arm), center + Vec2::splat(arm)], stroke);
    painter.line_segment([center + Vec2::new(-arm, arm), center + Vec2::new(arm, -arm)], stroke);
}

pub fn plus(painter: &Painter, center: Pos2, arm: f32, stroke: Stroke) {
    painter.line_segment([center - Vec2::new(arm, 0.0), center + Vec2::new(arm, 0.0)], stroke);
    painter.line_segment([center - Vec2::new(0.0, arm), center + Vec2::new(0.0, arm)], stroke);
}

pub fn progress_bar(painter: &Painter, track: Rect, fraction: f32, radius: f32, track_color: Color32, fill: Color32) {
    painter.rect_filled(track, radius, track_color);
    let mut filled = track;
    filled.set_width(track.width() * fraction.clamp(0.0, 1.0));
    painter.rect_filled(filled, radius, fill);
}

pub fn points_to_when_hovered(ui: &egui::Ui, response: &Response) {
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}
