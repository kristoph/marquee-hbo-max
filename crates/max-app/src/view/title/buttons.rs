use eframe::egui::{self, Id, Painter, Pos2, Rect, Response, Sense, Vec2};

use super::{icons, volume, TitleControl, SIDE_MARGIN};
use crate::{
    paint::{play_mark, points_to_when_hovered},
    player::NativePlayer,
    theme::{self, bold, DIM_TEXT, TEXT},
};

const BUTTON: f32 = 48.0;
const ROW_ABOVE_FOOT: f32 = 52.0;
const BACK_FROM_TOP: f32 = 72.0;
const BACK_FROM_LEFT: f32 = 88.0;
const BACK_ARM: f32 = 9.0;
const SKIP_FROM_CENTER: f32 = 78.0;
const FULL_SCREEN_FROM_RIGHT: f32 = 68.0;
const VOLUME_FROM_RIGHT: f32 = 124.0;
const PLAY_MARK_RADIUS: f32 = 13.0;
const HOVER_HALO: f32 = 0.16;
const CHIP_HEIGHT: f32 = 44.0;
const CHIP_PADDING: f32 = 56.0;
const CHIP_TEXT: f32 = 16.0;
const EPISODES: &str = "Episodes";

fn round_button(ui: &mut egui::Ui, center: Pos2, name: &str, paint: impl FnOnce(&Painter, Pos2)) -> Response {
    let response = ui.interact(Rect::from_center_size(center, Vec2::splat(BUTTON)), Id::new(("title-button", name)), Sense::click());
    points_to_when_hovered(ui, &response);
    if response.hovered() {
        ui.painter().circle_filled(center, BUTTON / 2.0, theme::white(HOVER_HALO));
    }
    paint(ui.painter(), center);
    response
}

pub(super) fn draw_back(ui: &mut egui::Ui, window: Rect) -> Option<TitleControl> {
    let center = window.min + Vec2::new(BACK_FROM_LEFT, BACK_FROM_TOP);
    round_button(ui, center, "back", |painter, center| icons::chevron_left(painter, center, BACK_ARM)).clicked().then_some(TitleControl::Close)
}

pub(super) fn draw_row(ui: &mut egui::Ui, window: Rect, player: &NativePlayer) -> Option<TitleControl> {
    let row = window.bottom() - ROW_ABOVE_FOOT;
    let at = |x: f32| Pos2::new(x, row);
    let middle = window.center().x;
    let playing = player.is_playing();

    let episodes = player.episode().is_some() && draw_chip(ui, Pos2::new(window.left() + SIDE_MARGIN, row), EPISODES);
    let back = round_button(ui, at(middle - SKIP_FROM_CENTER), "skip-back", |painter, center| icons::skip(painter, center, false));
    let play = round_button(ui, at(middle), "play", |painter, center| match playing {
        true => icons::pause(painter, center),
        false => play_mark(painter, center, PLAY_MARK_RADIUS, TEXT),
    });
    let forward = round_button(ui, at(middle + SKIP_FROM_CENTER), "skip-forward", |painter, center| icons::skip(painter, center, true));
    let volume = volume::draw(ui, at(window.right() - VOLUME_FROM_RIGHT), player);
    let full_screen = round_button(ui, at(window.right() - FULL_SCREEN_FROM_RIGHT), "full-screen", icons::full_screen);

    let pressed = [
        (episodes, TitleControl::OpenEpisodes),
        (back.clicked(), TitleControl::Skip { forward: false }),
        (play.clicked(), TitleControl::TogglePlaying),
        (forward.clicked(), TitleControl::Skip { forward: true }),
        (full_screen.clicked(), TitleControl::ToggleFullScreen),
    ];
    volume.or(pressed.into_iter().find_map(|(was_pressed, control)| was_pressed.then_some(control)))
}

/// A word in a pill that only shows its outline when pointed at, as the original's chips do.
pub(super) fn draw_chip(ui: &mut egui::Ui, left_center: Pos2, label: &str) -> bool {
    let text = ui.painter().layout_no_wrap(label.to_string(), bold(CHIP_TEXT), DIM_TEXT);
    let chip = Rect::from_min_size(left_center - Vec2::new(0.0, CHIP_HEIGHT / 2.0), Vec2::new(text.size().x + CHIP_PADDING, CHIP_HEIGHT));
    let response = ui.interact(chip, Id::new(("title-chip", label)), Sense::click());
    points_to_when_hovered(ui, &response);
    if response.hovered() {
        ui.painter().rect_filled(chip, CHIP_HEIGHT / 2.0, theme::white(HOVER_HALO));
    }
    ui.painter().galley(chip.center() - text.size() / 2.0, text, if response.hovered() { TEXT } else { DIM_TEXT });
    response.clicked()
}
