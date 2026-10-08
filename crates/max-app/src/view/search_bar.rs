use std::time::Instant;

use eframe::egui::{self, Align, Color32, Frame, Id, Key, Pos2, Rect, RichText, Sense, Stroke, TextEdit, Vec2};

use crate::{
    metrics::{header, search::BAR_HEIGHT, CORNER_RADIUS, MARGIN},
    model::Search,
    paint::{cross, points_to_when_hovered},
    theme::{regular, DIM_TEXT, TEXT},
    timing::SEARCH_DEBOUNCE,
};

const HINT: &str = "Find movies, shows, and more";
const FIELD_TEXT: f32 = 20.0;
const SPACE_BELOW_HEADER: f32 = 2.0;
const EMPTY_BAR: Color32 = Color32::from_rgb(18, 18, 18);
const FILLED_BAR: Color32 = Color32::from_rgb(40, 40, 40);
const GLASS_FROM_LEFT: f32 = 25.0;
const GLASS_RADIUS: f32 = 8.0;
const FIELD_INSET: f32 = 52.0;
const CLEAR_FROM_RIGHT: f32 = 26.0;
const CLEAR_TARGET: f32 = 36.0;
const CLEAR_ARM: f32 = 6.0;

pub(crate) fn draw(ui: &mut egui::Ui, search: &mut Search) {
    ui.add_space(header::HEIGHT + SPACE_BELOW_HEADER);
    let (strip, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), BAR_HEIGHT), Sense::hover());
    let bar = Rect::from_min_max(strip.min + Vec2::new(MARGIN, 0.0), strip.max - Vec2::new(MARGIN, 0.0));
    let painter = ui.painter().clone();
    painter.rect_filled(bar, CORNER_RADIUS, if search.text.is_empty() { EMPTY_BAR } else { FILLED_BAR });
    let stroke = Stroke::new(2.0, TEXT);
    let glass = Pos2::new(bar.left() + GLASS_FROM_LEFT, bar.center().y - 1.0);
    painter.circle_stroke(glass, GLASS_RADIUS, stroke);
    painter.line_segment([glass + Vec2::splat(6.0), glass + Vec2::splat(11.0)], stroke);

    let inner = Rect::from_min_max(Pos2::new(bar.left() + FIELD_INSET, bar.top()), Pos2::new(bar.right() - FIELD_INSET, bar.bottom()));
    let field = TextEdit::singleline(&mut search.text)
        // The automatic id shifts as rows load, which would drop the keyboard focus.
        .id(Id::new("search-field"))
        .hint_text(RichText::new(HINT).font(regular(FIELD_TEXT)).color(TEXT))
        .font(regular(FIELD_TEXT))
        .text_color(TEXT)
        .frame(Frame::NONE)
        .vertical_align(Align::Center)
        .desired_width(inner.width());
    let response = ui.put(inner, field);
    if std::mem::take(&mut search.focus_field) {
        response.request_focus();
    }
    if response.has_focus() && ui.input(|input| input.key_pressed(Key::ArrowDown)) {
        response.surrender_focus();
    }
    let mut edited = response.changed();
    if !search.text.is_empty() {
        let center = Pos2::new(bar.right() - CLEAR_FROM_RIGHT, bar.center().y);
        let clear = ui.interact(Rect::from_center_size(center, Vec2::splat(CLEAR_TARGET)), Id::new("search-clear"), Sense::click());
        points_to_when_hovered(ui, &clear);
        cross(&painter, center, CLEAR_ARM, Stroke::new(2.0, if clear.hovered() { TEXT } else { DIM_TEXT }));
        if clear.clicked() {
            search.text.clear();
            search.focus_field = true;
            edited = true;
        }
    }
    if edited {
        search.topic = None;
        search.edited = Some(Instant::now());
        ui.ctx().request_repaint_after(SEARCH_DEBOUNCE);
    }
}
