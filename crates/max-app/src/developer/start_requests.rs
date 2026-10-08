use std::time::{Duration, Instant};

use eframe::egui;

use crate::{
    app::App,
    intent::{Go, Intent, ProfileRequest, TileMenu, TileMenuRequest},
};

const FIRST_PAGE_SHOWN_FOR: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(100);

impl App {
    pub(crate) fn apply_start_requests(&mut self, ctx: &egui::Context, intent: &mut Intent) {
        if self.page.tabs.is_some() && self.navigation.loading.is_none() {
            intent.tab = intent.tab.or(self.developer.tab_to_open.take());
        }
        self.go_on_to_following_route(ctx, intent);
        if self.developer.open_episodes && self.playback.native.is_some() {
            self.developer.open_episodes = false;
            self.open_episodes(ctx);
        }
        if self.developer.open_profile_picker && self.fade.is_none() && !self.page.rows.is_empty() {
            self.developer.open_profile_picker = false;
            intent.profile = Some(ProfileRequest::OpenPicker);
        }
        if self.developer.open_tile_menu && self.fade.is_none() && !self.page.is_revealing_selection() {
            if let Some(artwork) = self.page.selected_artwork.get() {
                intent.tile_menu = Some(TileMenuRequest::Open(TileMenu::beside_artwork(self.page.selected, artwork)));
                self.developer.open_tile_menu = false;
            }
            ctx.request_repaint();
        }
    }

    fn go_on_to_following_route(&mut self, ctx: &egui::Context, intent: &mut Intent) {
        if self.page.rows.is_empty() || self.fade.is_some() {
            return;
        }
        let Some(following) = &mut self.developer.following_route else { return };
        ctx.request_repaint_after(POLL);
        if following.first_page_shown_at.get_or_insert_with(Instant::now).elapsed() >= FIRST_PAGE_SHOWN_FOR {
            intent.go = self.developer.following_route.take().map(|following| Go::Page(following.route));
        }
    }
}
