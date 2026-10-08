use std::thread;

use eframe::egui;
use max_api::{client::HOME_ROUTE, Error};

use super::App;
use crate::{
    intent::ProfileRequest,
    message::Message,
    model::{load_account, Chosen, ProfilePicker, ProfileSwitch, ScreenAccount},
};

impl App {
    pub(crate) fn handle_profile_request(&mut self, ctx: &egui::Context, request: ProfileRequest) {
        match request {
            ProfileRequest::OpenPicker => self.open_profile_picker(ctx),
            ProfileRequest::ClosePicker => self.profile_picker = None,
            ProfileRequest::Choose(profile) => self.choose_profile(ctx, profile),
            ProfileRequest::SubmitPin => self.submit_pin(ctx),
            ProfileRequest::CancelPin => self.profile_picker.iter_mut().for_each(|picker| picker.pin_entry = None),
            ProfileRequest::SignOut => self.sign_out(ctx),
        }
    }

    fn open_profile_picker(&mut self, ctx: &egui::Context) {
        let Some(client) = self.service.client() else { return };
        self.profile_picker = Some(ProfilePicker::default());
        self.browse_menu_open = false;
        self.tile_menu = None;
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let _ = sender.send(Message::Account(load_account(&client)));
            ctx.request_repaint();
        });
    }

    pub(crate) fn show_account(&mut self, account: Result<ScreenAccount, Error>) {
        match (account, &mut self.profile_picker) {
            (Ok(account), Some(picker)) => picker.account = Some(account),
            (Err(error), Some(_)) => {
                self.profile_picker = None;
                self.failed("Could not load profiles", &error);
            }
            (_, None) => {}
        }
    }

    fn choose_profile(&mut self, ctx: &egui::Context, index: usize) {
        match self.profile_picker.as_mut().and_then(|picker| picker.choose(index)) {
            Some(Chosen::AlreadyInUse) => self.profile_picker = None,
            Some(Chosen::Switch(switch)) => self.switch_profile(ctx, switch),
            Some(Chosen::AsksForItsPin) | None => {}
        }
    }

    fn submit_pin(&mut self, ctx: &egui::Context) {
        if let Some(switch) = self.profile_picker.as_mut().and_then(ProfilePicker::submit_pin) {
            self.switch_profile(ctx, switch);
        }
    }

    fn switch_profile(&mut self, ctx: &egui::Context, switch: ProfileSwitch) {
        let Some(client) = self.service.client() else { return };
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let switched = client.switch_profile(&switch.account, &switch.profile_id, switch.pin.as_deref());
            let _ = sender.send(Message::ProfileSwitched(switched));
            ctx.request_repaint();
        });
    }

    pub(crate) fn profile_switched(&mut self, ctx: &egui::Context, switched: Result<(), Error>) {
        match switched {
            Ok(()) => {
                self.start_afresh(ctx, HOME_ROUTE);
                self.load_chrome(ctx);
            }
            Err(error) => {
                self.profile_picker.iter_mut().for_each(|picker| picker.switching = false);
                self.failed("Could not switch profile", &error);
            }
        }
    }
}
