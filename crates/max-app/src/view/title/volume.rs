use eframe::egui::{self, Id, Pos2, Rect, Sense, Vec2};

use super::{icons, TitleControl};
use crate::{paint::points_to_when_hovered, player::NativePlayer, theme};

const BUTTON: f32 = 48.0;
const PANEL: Vec2 = Vec2::new(150.0, 40.0);
const PANEL_SHADE: f32 = 0.6;
const TRACK: Vec2 = Vec2::new(110.0, 4.0);
const TRACK_SHADE: f32 = 0.3;
const KNOB_RADIUS: f32 = 7.0;
const HOVER_HALO: f32 = 0.16;
const FADE_SECONDS: f32 = 0.15;

/// Pointing at the loudspeaker opens a slider beside it; clicking the loudspeaker mutes.
pub(super) fn draw(ui: &mut egui::Ui, center: Pos2, player: &NativePlayer) -> Option<TitleControl> {
    let button = Rect::from_center_size(center, Vec2::splat(BUTTON));
    let panel = Rect::from_min_size(Pos2::new(button.left() - PANEL.x, center.y - PANEL.y / 2.0), PANEL);
    let open_id = Id::new("title-volume-open");
    let was_open = ui.data(|data| data.get_temp::<bool>(open_id)).unwrap_or(false);
    let reach = if was_open { button.union(panel) } else { button };
    let open = ui.rect_contains_pointer(reach) || ui.ctx().is_being_dragged(Id::new("title-volume-slider"));
    ui.data_mut(|data| data.insert_temp(open_id, open));

    let response = ui.interact(button, Id::new("title-volume"), Sense::click());
    points_to_when_hovered(ui, &response);
    if response.hovered() {
        ui.painter().circle_filled(center, BUTTON / 2.0, theme::white(HOVER_HALO));
    }
    let heard = if player.is_muted() { 0.0 } else { player.volume() };
    icons::volume(ui.painter(), center, heard);

    let shown = ui.ctx().animate_bool_with_time(open_id.with("shown"), open, FADE_SECONDS);
    let set = if shown > 0.0 { draw_slider(ui, panel, heard, shown) } else { None };
    set.or(response.clicked().then_some(TitleControl::ToggleMuted))
}

fn draw_slider(ui: &mut egui::Ui, panel: Rect, heard: f32, shown: f32) -> Option<TitleControl> {
    let response = ui.interact(panel, Id::new("title-volume-slider"), Sense::click_and_drag());
    let painter = ui.painter().clone();
    painter.rect_filled(panel, PANEL.y / 2.0, egui::Color32::from_black_alpha((PANEL_SHADE * shown * 255.0) as u8));
    let track = Rect::from_center_size(panel.center(), TRACK);
    painter.rect_filled(track, TRACK.y / 2.0, theme::white(TRACK_SHADE * shown));
    let level = track.left() + track.width() * heard;
    painter.rect_filled(Rect::from_min_max(track.min, Pos2::new(level, track.bottom())), TRACK.y / 2.0, theme::white(shown));
    painter.circle_filled(Pos2::new(level, track.center().y), KNOB_RADIUS, theme::white(shown));

    let pressed_at = response.interact_pointer_pos().filter(|_| response.clicked() || response.dragged())?;
    Some(TitleControl::SetVolume(((pressed_at.x - track.left()) / track.width()).clamp(0.0, 1.0)))
}
