use std::thread;

use eframe::egui;
use max_api::{client::HOME_ROUTE, Error};

use super::App;
use crate::{
    intent::ProfileRequest,
    message::Message,
    model::{load_account, ScreenAccount},
};

pub struct PinEntry {
    pub profile: usize,
    pub digits: String,
    pub focus_field: bool,
}

#[derive(Default)]
pub struct ProfilePicker {
    pub account: Option<ScreenAccount>,
    pub pin_entry: Option<PinEntry>,
    pub switching: bool,
}

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
        let Some(picker) = &mut self.profile_picker else { return };
        let Some(profile) = picker.account.as_ref().and_then(|account| account.account.profiles.get(index)) else { return };
        if profile.selected {
            self.profile_picker = None;
        } else if profile.needs_pin {
            picker.pin_entry = Some(PinEntry { profile: index, digits: String::new(), focus_field: true });
        } else {
            self.switch_profile(ctx, index, None);
        }
    }

    fn submit_pin(&mut self, ctx: &egui::Context) {
        let Some(entry) = self.profile_picker.as_mut().and_then(|picker| picker.pin_entry.take()) else { return };
        self.switch_profile(ctx, entry.profile, Some(entry.digits));
    }

    fn switch_profile(&mut self, ctx: &egui::Context, index: usize, pin: Option<String>) {
        let Some(client) = self.service.client() else { return };
        let Some(picker) = &mut self.profile_picker else { return };
        let Some(account) = picker.account.as_ref().map(|account| account.account.clone()) else { return };
        let Some(profile_id) = account.profiles.get(index).map(|profile| profile.id.clone()) else { return };
        picker.switching = true;
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let switched = client.switch_profile(&account, &profile_id, pin.as_deref());
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
