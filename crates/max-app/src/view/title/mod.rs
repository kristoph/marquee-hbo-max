//! What is drawn over a title playing in the native player, laid out to the measurements of
//! the service's own web player.

mod buttons;
mod drawer;
mod footer;
mod icons;
mod prompts;
mod volume;

use std::time::Duration;

use eframe::egui::{self, Color32, Id, Pos2, Rect, Sense, Vec2};

use crate::{
    paint::{gradient, CornerColors},
    player::NativePlayer,
};

const SHOWN_AFTER_ACTIVITY: Duration = Duration::from_secs(3);
const FADE_SECONDS: f32 = 0.25;
const SIDE_MARGIN: f32 = 60.0;
const TOP_SHADE: f32 = 160.0;
const BOTTOM_SHADE: f32 = 320.0;
const SHADE: u8 = 200;

pub enum TitleControl {
    TogglePlaying,
    Skip { forward: bool },
    SeekTo(f64),
    ToggleMuted,
    SetVolume(f32),
    ToggleFullScreen,
    Close,
    OpenEpisodes,
    CloseEpisodes,
    PickSeason(u32),
    PlayEpisode { route: String, title: String },
    WatchCredits,
}

/// The video lies behind the window and shows wherever nothing is painted. The controls
/// stay for a few seconds after the pointer last moved, and while the title is paused.
pub(crate) fn draw(ui: &mut egui::Ui, player: &NativePlayer) -> Option<TitleControl> {
    let window = ui.max_rect();
    if ui.input(|input| input.pointer.delta() != Vec2::ZERO || input.pointer.any_pressed()) {
        player.show_controls();
    }
    let surface = ui.interact(window, Id::new("title-surface"), Sense::click());
    let drawer_rise = ui.ctx().animate_bool_with_time(Id::new("episodes-drawer"), player.episodes.is_some(), FADE_SECONDS);
    if let Some(panel) = player.episodes.as_ref().filter(|_| drawer_rise > 0.0) {
        let chosen = drawer::draw(ui, panel, player.video_id(), drawer_rise);
        return chosen.or(surface.clicked().then_some(TitleControl::CloseEpisodes));
    }

    let wanted = player.controls_active.get().elapsed() < SHOWN_AFTER_ACTIVITY || !player.is_playing();
    let opacity = ui.ctx().animate_bool_with_time(Id::new("title-controls"), wanted, FADE_SECONDS);
    let prompted = prompts::draw(ui, window, player);
    if opacity <= 0.0 {
        ui.ctx().set_cursor_icon(egui::CursorIcon::None);
        return prompted.or(surface.clicked().then_some(TitleControl::TogglePlaying));
    }
    ui.set_opacity(opacity);
    paint_shades(ui, window);
    let pressed = buttons::draw_back(ui, window).or(footer::draw(ui, window, player)).or(buttons::draw_row(ui, window, player));
    ui.set_opacity(1.0);
    prompted.or(pressed).or(surface.clicked().then_some(TitleControl::TogglePlaying))
}

fn paint_shades(ui: &egui::Ui, window: Rect) {
    let (shade, clear) = (Color32::from_black_alpha(SHADE), Color32::TRANSPARENT);
    let top = Rect::from_min_size(window.min, Vec2::new(window.width(), TOP_SHADE));
    gradient(ui.painter(), top, CornerColors::top_to_bottom(shade, clear));
    let bottom = Rect::from_min_max(Pos2::new(window.left(), window.bottom() - BOTTOM_SHADE), window.max);
    gradient(ui.painter(), bottom, CornerColors::top_to_bottom(clear, shade));
}
