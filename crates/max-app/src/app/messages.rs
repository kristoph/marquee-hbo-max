use eframe::egui;
use max_api::Error;

use super::App;
use crate::{message::Message, model::ScreenRow, transition::Arrival};

impl App {
    pub(crate) fn receive_messages(&mut self, ctx: &egui::Context) {
        while let Ok(message) = self.inbox.try_recv() {
            match message {
                Message::Chrome(chrome) => self.chrome = chrome,
                Message::PlayRoute { title, route: Ok(Some(route)) } => self.arrive(Arrival::Play { route, title }),
                Message::PlayRoute { title, route: Ok(None) } => self.arrive(Arrival::Failed(format!("{title} has nothing to play yet"))),
                Message::PlayRoute { title, route: Err(error) } => self.arrive_failing(&format!("Could not start {title}"), &error),
                Message::AccountChangeFailed(error) => self.failed("That didn't go through", &error),
                Message::SearchResults { query, results } => self.show_search_results(&query, results),
                Message::Preview { title, files } => self.start_preview(title, files),
                Message::PlayerLeft => {
                    println!("PLAYER closed");
                    self.close_player();
                }
                Message::Row(row) => self.place_row(row),
                Message::RowFailed { row, error } => {
                    self.place_row(row);
                    self.failed("Part of this page could not be loaded", &error);
                }
                Message::Episodes { show_id, season_number, loaded } => self.show_episodes(&show_id, season_number, loaded),
                Message::TitlePlayback { title, prepared } => self.open_native_player(title, prepared),
                Message::NextVideo { video_id, next } => self.offer_next_video(&video_id, next),
                Message::Account(account) => self.show_account(account),
                Message::ProfileSwitched(switched) => self.profile_switched(ctx, switched),
                Message::SignedOut => self.clear_web_data(ctx),
                Message::WebDataCleared => self.sign_in_wanted = true,
                Message::WebPlayerRequest(request) => self.web_player_made_a_request(ctx, request),
                Message::WebPlayerCookies(cookies) => self.check_web_player_session(ctx, cookies),
                Message::SignInChecked(checked) => self.finish_sign_in(ctx, checked),
                Message::Page { route, remember, page } => {
                    self.navigation.loading = None;
                    match page {
                        Ok(page) if page.rows.iter().any(ScreenRow::takes_space) => self.arrive(Arrival::Page { route, remember, page }),
                        Ok(_) | Err(Error::NotFound) => self.arrive(Arrival::Failed(format!("Nothing to show at {route}"))),
                        Err(error) => self.arrive_failing(&format!("Could not load {route}"), &error),
                    }
                }
            }
        }
    }
}
