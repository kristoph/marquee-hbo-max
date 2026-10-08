use max_api::Error;

use super::App;
use crate::{
    model::ScreenChrome,
    service::Service,
    transition::{Arrival, Fade, FadePhase},
};

impl App {
    /// Says what went wrong, unless it is that the session ended: nothing will work until the
    /// viewer signs in again, so that is offered instead.
    pub(crate) fn failed(&mut self, what: &str, error: &Error) {
        match error.is_signed_out() {
            true => self.session_ended(),
            false => self.say(format!("{what}: {error}")),
        }
    }

    /// As [`Self::failed`], for a failure to get to where a transition was heading.
    pub(crate) fn arrive_failing(&mut self, what: &str, error: &Error) {
        match error.is_signed_out() {
            true => self.session_ended(),
            false => self.arrive(Arrival::Failed(format!("{what}: {error}"))),
        }
    }

    fn session_ended(&mut self) {
        if !self.service.is_live() {
            return;
        }
        log::info!("SESSION ended; asking to sign in again");
        self.service = Service::SignedOut;
        self.clear_browsing();
        self.chrome = ScreenChrome::default();
        self.fade = Some(Fade::starting(FadePhase::Black));
        self.sign_in_wanted = true;
    }
}
