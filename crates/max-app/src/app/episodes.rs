use std::thread;

use eframe::egui;
use max_api::Error;

use super::App;
use crate::{
    loading::fetch_outstanding,
    message::Message,
    model::{EpisodesPanel, LoadedEpisodes, Outstanding, ScreenBuilder},
};

impl App {
    fn episodes_panel(&mut self) -> Option<&mut EpisodesPanel> {
        self.playback.native.as_mut()?.episodes.as_mut()
    }

    pub(crate) fn open_episodes(&mut self, ctx: &egui::Context) {
        let Some(player) = &mut self.playback.native else { return };
        let Some(episode) = player.episode() else { return };
        player.episodes = Some(EpisodesPanel::loading(episode.show_id, episode.season_number));
        self.load_episodes(ctx);
    }

    pub(crate) fn pick_season(&mut self, ctx: &egui::Context, season_number: u32) {
        let Some(panel) = self.episodes_panel() else { return };
        panel.season_number = Some(season_number);
        panel.episodes = None;
        self.load_episodes(ctx);
    }

    fn load_episodes(&mut self, ctx: &egui::Context) {
        let Some(client) = self.service.client() else { return };
        let Some(panel) = self.episodes_panel() else { return };
        let (show_id, season_number, season_parameters) = (panel.show_id.clone(), panel.season_number, panel.season_parameters());
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let built = client.episodes(&show_id, season_parameters.as_deref()).map(|row| {
                let mut builder = ScreenBuilder::new(true);
                let episodes = builder.row(&row).tiles;
                (LoadedEpisodes { seasons: row.filters, episodes }, builder.outstanding)
            });
            let (loaded, outstanding) = match built {
                Ok((loaded, outstanding)) => (Ok(loaded), outstanding),
                Err(error) => (Err(error), Outstanding::default()),
            };
            let _ = sender.send(Message::Episodes { show_id, season_number, loaded });
            ctx.request_repaint();
            fetch_outstanding(outstanding, Some(&client), &sender, &ctx);
        });
    }

    pub(crate) fn show_episodes(&mut self, show_id: &str, season_number: Option<u32>, loaded: Result<LoadedEpisodes, Error>) {
        let Some(panel) = self.episodes_panel().filter(|panel| panel.show_id == show_id && panel.season_number == season_number) else { return };
        match loaded {
            Ok(loaded) => panel.show(loaded),
            Err(error) => {
                self.playback.native.iter_mut().for_each(|player| player.episodes = None);
                self.failed("Could not load episodes", &error);
            }
        }
    }

    pub(crate) fn play_episode(&mut self, ctx: &egui::Context, route: String, title: String) {
        self.close_native_player();
        self.start_playing(ctx, route, title);
    }
}
