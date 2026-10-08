use std::{fs, sync::Arc, thread, time::Duration};

use eframe::egui;
use max_api::{
    client::{Client, HOME_ROUTE},
    session::{Session, WebCookie},
    Error,
};

use super::App;
use crate::{
    message::Message,
    model::ScreenChrome,
    service::Service,
    signin::{SignIn, WebPlayerRequest},
    transition::{Fade, FadePhase},
    web,
};

const WAITING_TO_LOAD_POLL: Duration = Duration::from_millis(50);

impl App {
    pub(crate) fn sign_out(&mut self, ctx: &egui::Context) {
        let client = self.service.client();
        self.service = Service::SignedOut;
        self.clear_browsing();
        self.chrome = ScreenChrome::default();
        self.fade = Some(Fade::starting(FadePhase::Black));
        if let Err(error) = fs::remove_file(&self.session_path) {
            log::warn!("the saved session could not be removed: {error}");
        }
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            if let Some(Err(error)) = client.map(|client| client.sign_out()) {
                log::warn!("the service did not confirm signing out: {error}");
            }
            let _ = sender.send(Message::SignedOut);
            ctx.request_repaint();
        });
    }

    pub(crate) fn clear_web_data(&mut self, ctx: &egui::Context) {
        let (sender, repaint) = (self.sender.clone(), ctx.clone());
        let cleared = web::cookies::clear_all(move || {
            let _ = sender.send(Message::WebDataCleared);
            repaint.request_repaint();
        });
        if let Err(error) = cleared {
            log::warn!("web data could not be cleared: {error}");
            self.sign_in_wanted = true;
        }
    }

    pub(crate) fn open_sign_in(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        self.sign_in_wanted = false;
        let (sender, repaint) = (self.sender.clone(), ctx.clone());
        let on_request = move |request: WebPlayerRequest| {
            let _ = sender.send(Message::WebPlayerRequest(request));
            repaint.request_repaint();
        };
        let cookies_to_start_with =
            self.developer.session_to_sign_in_with.as_ref().and_then(|path| Session::load(path).ok()).map(|session| session.web_cookies);
        match SignIn::embed(frame, ctx.content_rect(), &cookies_to_start_with.unwrap_or_default(), on_request) {
            Ok(sign_in) => self.sign_in = Some(sign_in),
            Err(error) => self.say(format!("Could not open the sign-in page: {error}")),
        }
    }

    pub(crate) fn tend_sign_in(&mut self, ui: &mut egui::Ui) {
        let Some(sign_in) = &mut self.sign_in else { return };
        sign_in.keep_filling(ui.max_rect());
        if sign_in.is_waiting_to_load() {
            sign_in.load_when_cookies_are_stored();
            ui.ctx().request_repaint_after(WAITING_TO_LOAD_POLL);
        }
    }

    pub(crate) fn web_player_made_a_request(&mut self, ctx: &egui::Context, request: WebPlayerRequest) {
        let Some(sign_in) = self.sign_in.as_mut().filter(|sign_in| sign_in.request.is_none() && !sign_in.checking) else { return };
        sign_in.request = Some(request);
        sign_in.checking = true;
        let (sender, repaint) = (self.sender.clone(), ctx.clone());
        sign_in.read_cookies(move |cookies| {
            let _ = sender.send(Message::WebPlayerCookies(cookies));
            repaint.request_repaint();
        });
    }

    /// A visitor who has not signed in has a session too, so the one the web player holds is
    /// only taken once the service says it belongs to an account.
    pub(crate) fn check_web_player_session(&mut self, ctx: &egui::Context, cookies: Vec<WebCookie>) {
        let Some(request) = self.sign_in.as_mut().and_then(|sign_in| sign_in.request.take()) else { return };
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let checked = Session::from_web_player(request.home_url, request.headers, cookies).and_then(|session| {
                match Client::new(session.clone()).account()?.signed_in {
                    true => Ok(session),
                    false => Err(Error::SignedOut),
                }
            });
            let _ = sender.send(Message::SignInChecked(checked));
            ctx.request_repaint();
        });
    }

    pub(crate) fn finish_sign_in(&mut self, ctx: &egui::Context, checked: Result<Session, Error>) {
        let session = match checked {
            Ok(session) => session,
            Err(reason) => {
                log::info!("SIGN-IN not complete: {reason}");
                return self.sign_in.iter_mut().for_each(SignIn::show_page_again);
            }
        };
        if let Err(error) = session.save(&self.session_path) {
            log::warn!("the session could not be saved: {error}");
        }
        log::info!("SIGN-IN complete");
        self.sign_in = None;
        self.service = Service::Live(Arc::new(Client::new(session)));
        self.start_afresh(ctx, HOME_ROUTE);
        self.load_chrome(ctx);
    }
}
