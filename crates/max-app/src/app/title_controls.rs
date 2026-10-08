use eframe::egui::{self, Key, ViewportCommand};

use super::App;
use crate::view::title::TitleControl;

impl App {
    pub(crate) fn apply_title_control(&mut self, ctx: &egui::Context, control: TitleControl) {
        let Some(player) = &mut self.playback.native else { return };
        match control {
            TitleControl::TogglePlaying => player.toggle_playing(),
            TitleControl::SeekTo(position) => player.seek(position),
            TitleControl::Skip { forward } => player.skip(forward),
            TitleControl::ToggleMuted => player.toggle_muted(),
            TitleControl::SetVolume(volume) => player.set_volume(volume),
            TitleControl::ToggleFullScreen => toggle_full_screen(ctx),
            TitleControl::CloseEpisodes => player.episodes = None,
            TitleControl::OpenEpisodes => self.open_episodes(ctx),
            TitleControl::PickSeason(season_number) => self.pick_season(ctx, season_number),
            TitleControl::PlayEpisode { route, title } => self.play_episode(ctx, route, title),
            TitleControl::WatchCredits => player.watch_credits(),
            TitleControl::Close => self.close_native_player(),
        }
    }

    pub(crate) fn title_control_from_keys(&self, ctx: &egui::Context) -> Option<TitleControl> {
        let player = self.playback.native.as_ref()?;
        if ctx.input(|input| !input.keys_down.is_empty()) {
            player.show_controls();
        }
        let pressed = |key| ctx.input(|input| input.key_pressed(key));
        if pressed(Key::Space) {
            Some(TitleControl::TogglePlaying)
        } else if pressed(Key::ArrowRight) {
            Some(TitleControl::Skip { forward: true })
        } else if pressed(Key::ArrowLeft) {
            Some(TitleControl::Skip { forward: false })
        } else {
            None
        }
    }
}

fn toggle_full_screen(ctx: &egui::Context) {
    let full_screen = ctx.input(|input| input.viewport().fullscreen.unwrap_or(false));
    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full_screen));
}
