use eframe::egui::{self, CentralPanel, Color32, Frame, RichText};

use super::App;
use crate::{
    intent::{Go, Intent, TileMenuRequest},
    theme, view,
};

const FAILURE_TEXT: f32 = 22.0;

impl eframe::App for App {
    /// The window is see-through only where a title's video lies behind it.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let alpha = if self.playback.native.is_some() { 0.0 } else { 1.0 };
        [0.0, 0.0, 0.0, alpha]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.remember_window_view(frame);
        self.receive_messages(&ctx);
        self.run_fade(&ctx);
        self.run_search(&ctx);
        self.hero.in_view.set(false);

        let mut intent = Intent::default();
        self.run_benchmark(&ctx, frame);
        self.handle_keys(&ctx, &mut intent);
        self.run_walkthrough(&ctx, &mut intent);
        self.take_screenshot(&ctx);

        let behind_everything = if self.playback.native.is_some() { Color32::TRANSPARENT } else { theme::BACKGROUND };
        CentralPanel::default().frame(Frame::new().fill(behind_everything)).show(ui, |ui| {
            if self.sign_in.is_some() {
                self.tend_sign_in(ui);
            } else if self.playback.web.is_some() {
                self.tend_player(ui);
            } else if self.playback.native.is_some() {
                self.tend_native_player(ui);
            } else if self.profile_picker.is_some() {
                let mut picker = self.profile_picker.take();
                if let Some(picker) = &mut picker {
                    view::profiles::draw(ui, picker, &mut intent);
                }
                self.profile_picker = picker;
                self.draw_toast(ui, ui.max_rect());
            } else if self.page.rows.is_empty() {
                self.draw_failure(ui);
            } else {
                self.draw_page(ui, &mut intent);
            }
        });
        self.run_preview(&ctx);
        self.run_hero_timer(&ctx);
        self.draw_fade(&ctx);
        self.apply_start_requests(&ctx, &mut intent);

        let in_transition = self.fade.is_some() && !self.playback.is_open();
        if in_transition {
            intent = Intent::default();
        }
        self.apply(&ctx, intent);
        if let Some(playback) = self.playback.to_open.take() {
            self.open_player(&ctx, frame, playback);
        }
        if self.sign_in_wanted {
            self.open_sign_in(&ctx, frame);
        }
    }
}

impl App {
    fn draw_failure(&self, ui: &mut egui::Ui) {
        if self.navigation.loading.is_some() {
            return;
        }
        if let Some(toast) = &self.toast {
            ui.centered_and_justified(|ui| ui.label(RichText::new(&toast.message).font(theme::regular(FAILURE_TEXT)).color(Color32::LIGHT_RED)));
        }
    }

    fn apply(&mut self, ctx: &egui::Context, intent: Intent) {
        if let Some(selection) = intent.select {
            self.page.selected = selection;
            if selection.row == 0 {
                self.set_hero(selection.column);
            }
        }
        self.swipe_hero(ctx, &intent);
        if let Some(open) = intent.browse_menu {
            self.browse_menu_open = open;
        }
        if let Some(index) = intent.tab {
            self.open_tab(ctx, index);
        }
        if intent.toggle_preview_sound {
            self.toggle_preview_sound();
        }
        match intent.tile_menu {
            Some(TileMenuRequest::Open(menu)) => {
                self.tile_menu = Some(menu).filter(|menu| self.page.tile_at(menu.tile).is_some_and(|tile| !tile.detail.menu.is_empty()));
            }
            Some(TileMenuRequest::Close) => self.tile_menu = None,
            None => {}
        }
        if let Some(request) = intent.profile {
            self.handle_profile_request(ctx, request);
        }
        if let Some(change) = intent.account_change {
            self.tile_menu = None;
            self.change_account(ctx, change);
        }
        if let Some(go) = intent.go {
            let closed_tile_menu = self.tile_menu.take().is_some();
            let only_closes_tile_menu = matches!(go, Go::Back) && closed_tile_menu && !self.playback.is_open() && !self.browse_menu_open;
            if !only_closes_tile_menu {
                self.go(ctx, go);
            }
        }
    }

    fn leave_native_player_or_its_episodes(&mut self) {
        match self.playback.native.as_mut().and_then(|player| player.episodes.take()) {
            Some(_) => {}
            None => self.close_native_player(),
        }
    }

    fn go(&mut self, ctx: &egui::Context, go: Go) {
        match go {
            Go::Page(route) => {
                self.browse_menu_open = false;
                if route != self.navigation.route && self.navigation.loading.is_none() {
                    self.go_to(ctx, route, true);
                }
            }
            Go::Play { route, title } => self.start_playing(ctx, route, title),
            Go::Back if self.playback.web.is_some() => self.close_player(),
            Go::Back if self.playback.native.is_some() => self.leave_native_player_or_its_episodes(),
            Go::Back if self.profile_picker.is_some() => self.profile_picker = None,
            Go::Back if self.browse_menu_open => self.browse_menu_open = false,
            Go::Back => self.go_back(ctx),
        }
    }
}
