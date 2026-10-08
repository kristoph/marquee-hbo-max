use eframe::egui::{
    self, scroll_area::ScrollBarVisibility, Color32, FontId, Id, Image, Pos2, Rect, ScrollArea, Sense, Stroke, StrokeKind, UiBuilder, Vec2,
};

use super::{TitleControl, SIDE_MARGIN};
use crate::{
    metrics::{rail::TILE_CORNER_RADIUS, SMALL_TEXT},
    model::{EpisodesPanel, ScreenTile},
    paint::{cross, gradient, play_mark, points_to_when_hovered, progress_bar, CornerColors, WrappedText},
    theme::{self, bold, regular, DIM_TEXT, PLACEHOLDER, TEXT},
};

const HEIGHT: f32 = 491.0;
const FADE_IN_HEIGHT: f32 = 110.0;
const SHADE: u8 = 232;
const CLOSE_FROM_LEFT: f32 = 28.0;
const CLOSE_FROM_TOP: f32 = 28.0;
const CLOSE_TARGET: f32 = 48.0;
const CLOSE_ARM: f32 = 7.0;
const CHIPS_FROM_TOP: f32 = 78.0;
const CHIP_HEIGHT: f32 = 44.0;
const CHIP_PADDING: f32 = 48.0;
const CHIP_GAP: f32 = 12.0;
const CHIP_TEXT: f32 = 16.0;
const CARDS_FROM_TOP: f32 = 120.0;
const CARD: Vec2 = Vec2::new(355.0, 200.0);
const CARD_GAP: f32 = 20.0;
const CAPTION_HEIGHT: f32 = 110.0;
const TITLE_BELOW_PICTURE: f32 = 6.0;
const LINE_SPACING: f32 = 21.0;
const DESCRIPTION_LINES: usize = 3;
const FACTS_SEPARATOR: &str = "   ";
const MARK_FROM_LEFT: f32 = 16.0;
const MARK_ABOVE_FOOT: f32 = 26.0;
const PLAY_MARK_RADIUS: f32 = 9.0;
const NOW_PLAYING_BARS: [f32; 3] = [10.0, 16.0, 7.0];
const PROGRESS_HEIGHT: f32 = 4.0;
const HOVER_RING: f32 = 2.0;

/// Rises from the foot of the window with the season's episodes side by side.
pub(super) fn draw(ui: &mut egui::Ui, panel: &EpisodesPanel, playing_video_id: &str, rise: f32) -> Option<TitleControl> {
    let window = ui.max_rect();
    let top = window.bottom() - HEIGHT * rise;
    let area = Rect::from_min_size(Pos2::new(window.left(), top), Vec2::new(window.width(), HEIGHT));
    ui.interact(area, Id::new("episodes-drawer-area"), Sense::click());
    let shade = Color32::from_black_alpha(SHADE);
    gradient(
        ui.painter(),
        Rect::from_min_size(area.min, Vec2::new(area.width(), FADE_IN_HEIGHT)),
        CornerColors::top_to_bottom(Color32::TRANSPARENT, shade),
    );
    ui.painter().rect_filled(Rect::from_min_max(area.min + Vec2::new(0.0, FADE_IN_HEIGHT), window.max), 0.0, shade);

    let mut control = draw_close(ui, area).or(draw_chips(ui, panel, area));
    let Some(episodes) = &panel.episodes else { return control };
    let cards = Rect::from_min_size(Pos2::new(area.left(), top + CARDS_FROM_TOP), Vec2::new(area.width(), CARD.y + CAPTION_HEIGHT));
    ui.scope_builder(UiBuilder::new().max_rect(cards), |ui| {
        ScrollArea::horizontal()
            .id_salt(("episodes", panel.season_number))
            .auto_shrink(false)
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = CARD_GAP;
                    ui.add_space(SIDE_MARGIN);
                    for (index, episode) in episodes.iter().enumerate() {
                        let now_playing = episode.route.as_deref().is_some_and(|route| route.contains(playing_video_id));
                        if draw_card(ui, index, episode, now_playing) && !now_playing {
                            control = episode.route.clone().map(|route| TitleControl::PlayEpisode { route, title: episode.title.clone() });
                        }
                    }
                    ui.add_space(SIDE_MARGIN - CARD_GAP);
                });
            });
    });
    control
}

fn draw_close(ui: &mut egui::Ui, area: Rect) -> Option<TitleControl> {
    let center = area.min + Vec2::new(CLOSE_FROM_LEFT, CLOSE_FROM_TOP);
    let response = ui.interact(Rect::from_center_size(center, Vec2::splat(CLOSE_TARGET)), Id::new("episodes-close"), Sense::click());
    points_to_when_hovered(ui, &response);
    cross(ui.painter(), center, CLOSE_ARM, Stroke::new(2.0, if response.hovered() { TEXT } else { DIM_TEXT }));
    response.clicked().then_some(TitleControl::CloseEpisodes)
}

/// "Episodes" for a series with one season, otherwise a chip for each season.
fn draw_chips(ui: &mut egui::Ui, panel: &EpisodesPanel, area: Rect) -> Option<TitleControl> {
    let mut picked = None;
    let mut left = area.left() + SIDE_MARGIN;
    let middle = area.top() + CHIPS_FROM_TOP;
    let mut chip = |ui: &mut egui::Ui, label: String, selected: bool| {
        let ink = if selected { TEXT } else { DIM_TEXT };
        let text = ui.painter().layout_no_wrap(label.clone(), bold(CHIP_TEXT), ink);
        let pill = Rect::from_min_size(Pos2::new(left, middle - CHIP_HEIGHT / 2.0), Vec2::new(text.size().x + CHIP_PADDING, CHIP_HEIGHT));
        let response = ui.interact(pill, Id::new(("episodes-chip", label)), Sense::click());
        points_to_when_hovered(ui, &response);
        if selected {
            ui.painter().rect_stroke(pill, CHIP_HEIGHT / 2.0, Stroke::new(2.0, TEXT), StrokeKind::Inside);
        } else if response.hovered() {
            ui.painter().rect_filled(pill, CHIP_HEIGHT / 2.0, theme::white(0.16));
        }
        ui.painter().galley(pill.center() - text.size() / 2.0, text, ink);
        left = pill.right() + CHIP_GAP;
        response.clicked() && !selected
    };
    if panel.seasons.len() < 2 {
        chip(ui, "Episodes".to_string(), true);
        return None;
    }
    for season in &panel.seasons {
        if chip(ui, format!("Season {}", season.label), panel.is_selected(season)) {
            picked = season.label.parse().ok().map(TitleControl::PickSeason);
        }
    }
    picked
}

fn draw_card(ui: &mut egui::Ui, index: usize, episode: &ScreenTile, now_playing: bool) -> bool {
    let (card, response) = ui.allocate_exact_size(Vec2::new(CARD.x, CARD.y + CAPTION_HEIGHT), Sense::click());
    if !ui.is_rect_visible(card) {
        return false;
    }
    points_to_when_hovered(ui, &response);
    let picture = Rect::from_min_size(card.min, CARD);
    match episode.artwork.as_ref().and_then(|artwork| artwork.uri()) {
        Some(uri) => {
            Image::new(uri).corner_radius(TILE_CORNER_RADIUS).paint_at(ui, picture);
        }
        None => {
            ui.painter().rect_filled(picture, TILE_CORNER_RADIUS, PLACEHOLDER);
        }
    }
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_stroke(picture, TILE_CORNER_RADIUS, Stroke::new(HOVER_RING, TEXT), StrokeKind::Outside);
    }
    if let Some(progress) = episode.detail.progress {
        let track = Rect::from_min_max(Pos2::new(picture.left(), picture.bottom() - PROGRESS_HEIGHT), picture.max);
        progress_bar(painter, track, progress, 0.0, Color32::from_gray(70), TEXT);
    }
    let mark = Pos2::new(picture.left() + MARK_FROM_LEFT + PLAY_MARK_RADIUS, picture.bottom() - MARK_ABOVE_FOOT);
    match now_playing {
        true => paint_now_playing(painter, mark),
        false => play_mark(painter, mark, PLAY_MARK_RADIUS, TEXT),
    }

    let number = episode.detail.season_and_episode.map_or(index as u32 + 1, |(_, number)| number);
    let paint_line = |text: &str, font: FontId, lines: usize, top: f32| {
        WrappedText { text, font, color: DIM_TEXT, width: CARD.x, lines }.paint(painter, Pos2::new(card.left(), top));
    };
    let title_top = picture.bottom() + TITLE_BELOW_PICTURE;
    paint_line(&format!("{number}. {}", episode.title), bold(SMALL_TEXT), 1, title_top);
    paint_line(&episode.detail.facts.join(FACTS_SEPARATOR), regular(SMALL_TEXT), 1, title_top + LINE_SPACING);
    if let Some(description) = &episode.detail.description {
        paint_line(description, regular(SMALL_TEXT), DESCRIPTION_LINES, title_top + LINE_SPACING * 2.0 - 1.0);
    }
    response.clicked()
}

fn paint_now_playing(painter: &egui::Painter, center: Pos2) {
    for (index, height) in NOW_PLAYING_BARS.iter().enumerate() {
        let left = center.x - 8.0 + index as f32 * 6.0;
        painter.rect_filled(Rect::from_min_max(Pos2::new(left, center.y + 8.0 - height), Pos2::new(left + 3.0, center.y + 8.0)), 0.0, TEXT);
    }
}
