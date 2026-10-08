use std::{thread, time::Duration};

use eframe::egui;
use max_api::{cms::NextVideo, playback::TitlePlayback, Error};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use super::App;
use crate::{
    message::Message,
    player::{NativePlayer, PendingPlayback},
    view::{self, title::TitleControl},
};

const TEND_EVERY: Duration = Duration::from_millis(250);

impl App {
    pub(crate) fn switch_player(&mut self) {
        self.playback.choice = self.playback.choice.other();
        self.say(self.playback.choice.description().to_string());
    }

    pub(crate) fn remember_window_view(&mut self, frame: &eframe::Frame) {
        self.window_view = match frame.window_handle().map(|handle| handle.as_raw()) {
            Ok(RawWindowHandle::AppKit(handle)) => Some(handle.ns_view),
            _ => None,
        };
    }

    pub(crate) fn prepare_native_player(&mut self, ctx: &egui::Context, playback: PendingPlayback) {
        let Some(client) = self.service.client() else { return self.native_player_failed("not signed in") };
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let prepared = client.title_playback(&playback.route);
            let _ = sender.send(Message::TitlePlayback { title: playback.title, prepared });
            ctx.request_repaint();
        });
    }

    pub(crate) fn open_native_player(&mut self, title: String, prepared: Result<TitlePlayback, Error>) {
        let playback = match prepared {
            Ok(playback) => playback,
            Err(error) if error.is_signed_out() => return self.failed("Could not play this", &error),
            Err(error) => return self.native_player_failed(&error.to_string()),
        };
        let (Some(client), Some(view)) = (self.service.client(), self.window_view) else {
            return self.native_player_failed("the window has no view to show video in");
        };
        match NativePlayer::open(client, title, playback, view, self.playback.sound) {
            Ok(player) => {
                println!("NATIVE PLAYER opened");
                self.find_next_video(&player);
                self.playback.native = Some(player);
                // The video lies behind what the app draws, so the black of the transition must go.
                self.fade = None;
            }
            Err(error) => self.native_player_failed(&error.to_string()),
        }
    }

    fn find_next_video(&self, player: &NativePlayer) {
        let Some(client) = self.service.client().filter(|_| player.episode().is_some()) else { return };
        let (sender, video_id) = (self.sender.clone(), player.video_id().to_string());
        thread::spawn(move || {
            let next = client.next_video(&video_id);
            let _ = sender.send(Message::NextVideo { video_id, next });
        });
    }

    pub(crate) fn offer_next_video(&mut self, video_id: &str, next: Result<Option<NextVideo>, Error>) {
        let Some(player) = self.playback.native.as_mut().filter(|player| player.video_id() == video_id) else { return };
        match next {
            Ok(next) => player.next = next,
            Err(error) => eprintln!("the next episode could not be found: {error}"),
        }
    }

    fn native_player_failed(&mut self, reason: &str) {
        println!("NATIVE PLAYER failed: {reason}");
        self.playback.native = None;
        self.say(format!("The native player could not play this: {reason}"));
        self.fade_in();
    }

    pub(crate) fn close_native_player(&mut self) {
        println!("NATIVE PLAYER closed");
        if let Some(player) = self.playback.native.take() {
            player.report_progress();
        }
        self.forget_visited_pages();
        self.fade_in();
    }

    pub(crate) fn tend_native_player(&mut self, ui: &mut egui::Ui) {
        let Some(player) = &mut self.playback.native else { return };
        ui.ctx().request_repaint_after(TEND_EVERY);
        player.report_progress_when_due();
        player.log_progress_when_due();
        self.playback.sound = player.sound();
        let (failure, finished) = (player.failure(), player.finished());
        let next_due = player.next_when_due().map(|next| TitleControl::PlayEpisode { route: next.route.clone(), title: next.name.clone() });

        let from_keys = self.title_control_from_keys(ui.ctx());
        let from_pointer = self.playback.native.as_ref().and_then(|player| view::title::draw(ui, player));
        match (failure, from_pointer.or(from_keys)) {
            (Some(reason), _) => self.native_player_failed(&reason),
            (None, Some(control)) => self.apply_title_control(ui.ctx(), control),
            (None, None) => match next_due {
                Some(play_next) => self.apply_title_control(ui.ctx(), play_next),
                None if finished => self.apply_title_control(ui.ctx(), TitleControl::Close),
                None => {}
            },
        }
    }
}
