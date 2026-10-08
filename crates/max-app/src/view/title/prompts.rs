use eframe::egui::{self, Align2, Color32, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2};
use max_api::cms::NextVideo;

use super::{TitleControl, SIDE_MARGIN};
use crate::{
    paint::{play_mark, points_to_when_hovered},
    player::{NativePlayer, Prompt},
    theme::{bold, regular, TEXT},
};

const ABOVE_FOOT: f32 = 214.0;
const HEIGHT: f32 = 48.0;
const PADDING: f32 = 26.0;
const GAP: f32 = 14.0;
const LABEL_TEXT: f32 = 17.0;
const CAPTION_TEXT: f32 = 16.0;
const CAPTION_ABOVE_BUTTONS: f32 = 14.0;
const PLAY_MARK_ROOM: f32 = 26.0;
const PLAY_MARK_RADIUS: f32 = 7.0;
const BACKING: u8 = 150;
const HOVERED_BACKING: u8 = 60;
const NOT_YET_COUNTED_DOWN: Color32 = Color32::from_gray(170);
const NEXT_EPISODE: &str = "Next Episode";
const WATCH_CREDITS: &str = "Watch Credits";

/// These stay up while the controls are hidden: they are offered by the title, not asked for.
pub(super) fn draw(ui: &mut egui::Ui, window: Rect, player: &NativePlayer) -> Option<TitleControl> {
    let right_center = Pos2::new(window.right() - SIDE_MARGIN, window.bottom() - ABOVE_FOOT);
    match player.prompt()? {
        Prompt::Skip { label, to_seconds } => outlined_button(ui, right_center, label).clicked().then_some(TitleControl::SeekTo(to_seconds)),
        Prompt::UpNext { next, counted_down } => {
            let play_next = next_button(ui, right_center, counted_down);
            let watch_credits = outlined_button(ui, play_next.rect.left_center() - Vec2::new(GAP, 0.0), WATCH_CREDITS);
            let caption_at = play_next.rect.right_top() - Vec2::new(0.0, CAPTION_ABOVE_BUTTONS);
            ui.painter().text(caption_at, Align2::RIGHT_BOTTOM, caption(next), regular(CAPTION_TEXT), TEXT);
            match (play_next.clicked(), watch_credits.clicked()) {
                (true, _) => Some(TitleControl::PlayEpisode { route: next.route.clone(), title: next.name.clone() }),
                (_, true) => Some(TitleControl::WatchCredits),
                _ => None,
            }
        }
    }
}

fn caption(next: &NextVideo) -> String {
    match next.season_and_episode {
        Some((season, episode)) => format!("S{season} E{episode}:  {}", next.name),
        None => next.name.clone(),
    }
}

fn pill(ui: &mut egui::Ui, right_center: Pos2, content_width: f32, name: &str) -> Response {
    let size = Vec2::new(content_width + PADDING * 2.0, HEIGHT);
    let response =
        ui.interact(Rect::from_min_size(right_center - Vec2::new(size.x, HEIGHT / 2.0), size), Id::new(("title-prompt", name)), Sense::click());
    points_to_when_hovered(ui, &response);
    response
}

fn outlined_button(ui: &mut egui::Ui, right_center: Pos2, label: &str) -> Response {
    let text = ui.painter().layout_no_wrap(label.to_string(), bold(LABEL_TEXT), TEXT);
    let response = pill(ui, right_center, text.size().x, label);
    let fill = if response.hovered() { Color32::from_white_alpha(HOVERED_BACKING) } else { Color32::from_black_alpha(BACKING) };
    ui.painter().rect(response.rect, HEIGHT / 2.0, fill, Stroke::new(1.5, TEXT), StrokeKind::Inside);
    ui.painter().galley(response.rect.center() - text.size() / 2.0, text, TEXT);
    response
}

/// The button fills with white from the left as the wait for the next episode runs out.
fn next_button(ui: &mut egui::Ui, right_center: Pos2, counted_down: f32) -> Response {
    let text = ui.painter().layout_no_wrap(NEXT_EPISODE.to_string(), bold(LABEL_TEXT), Color32::BLACK);
    let response = pill(ui, right_center, text.size().x + PLAY_MARK_ROOM, NEXT_EPISODE);
    let button = response.rect;
    let counted = Rect::from_min_max(button.min, Pos2::new(button.left() + button.width() * counted_down, button.bottom()));
    ui.painter().rect_filled(button, HEIGHT / 2.0, NOT_YET_COUNTED_DOWN);
    ui.painter().with_clip_rect(counted).rect_filled(button, HEIGHT / 2.0, Color32::WHITE);
    play_mark(ui.painter(), Pos2::new(button.left() + PADDING + PLAY_MARK_RADIUS, button.center().y), PLAY_MARK_RADIUS, Color32::BLACK);
    ui.painter().galley(Pos2::new(button.left() + PADDING + PLAY_MARK_ROOM, button.center().y - text.size().y / 2.0), text, Color32::BLACK);
    response
}
