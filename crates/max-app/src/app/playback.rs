use std::{thread, time::Duration};

use eframe::egui;
use max_api::Error;

use super::App;
use crate::player::PendingPlayback;
use crate::{
    message::Message,
    player::{Player, PlayerChoice, PlayerOptions},
    routes,
    transition::{Arrival, Fade, FadePhase},
    Failure,
};

const WAITING_TO_LOAD_POLL: Duration = Duration::from_millis(50);

impl App {
    /// A title's detail route is first resolved to the route its own play button leads to, so
    /// that the player opens straight on the video and never on a page of the web app.
    pub(crate) fn start_playing(&mut self, ctx: &egui::Context, route: String, title: String) {
        let fade = self.fade.insert(Fade::starting(FadePhase::Out));
        if routes::starts_playback(&route) {
            return fade.deliver(Arrival::Play { route, title });
        }
        let Some(client) = self.service.client() else { return fade.deliver(Arrival::Failed("Not signed in".to_string())) };
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let found = client.play_route(&route).and_then(|found| found.ok_or(Error::Lacks("anything to play")));
            let _ = sender.send(Message::PlayRoute { title, route: found });
            ctx.request_repaint();
        });
    }

    pub(crate) fn open_player(&mut self, ctx: &egui::Context, frame: &eframe::Frame, playback: PendingPlayback) {
        self.playback.web = None;
        self.playback.native = None;
        if self.playback.choice == PlayerChoice::Native {
            return self.prepare_native_player(ctx, playback);
        }
        match self.embed_player(ctx, frame, &playback) {
            Ok(player) => self.playback.web = Some(player),
            Err(error) => {
                self.say(format!("Could not open the player: {error}"));
                self.fade_in();
            }
        }
    }

    fn embed_player(&self, ctx: &egui::Context, frame: &eframe::Frame, playback: &PendingPlayback) -> Result<Player, Failure> {
        if !routes::starts_playback(&playback.route) {
            return Err(format!("{} is not a video", playback.title).into());
        }
        let client = self.service.client().ok_or("not signed in")?;
        let (sender, repaint) = (self.sender.clone(), ctx.clone());
        let on_close = move || {
            let _ = sender.send(Message::PlayerLeft);
            repaint.request_repaint();
        };
        let options =
            PlayerOptions { session: client.session(), route: &playback.route, bounds: ctx.content_rect(), muted: self.playback.sound.muted };
        Player::embed(frame, options, on_close)
    }

    pub(crate) fn close_player(&mut self) {
        self.playback.web = None;
        self.fade_in();
    }

    pub(crate) fn tend_player(&mut self, ui: &mut egui::Ui) {
        let Some(player) = &mut self.playback.web else { return };
        player.keep_filling(ui.max_rect());
        player.load_when_signed_in();
        if player.is_waiting_to_load() {
            ui.ctx().request_repaint_after(WAITING_TO_LOAD_POLL);
        }
    }
}
