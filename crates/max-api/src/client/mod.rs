mod account;
mod chrome;
mod episodes;
mod http;
mod next;
mod pages;
mod preview;
mod profiles;
mod search;

use std::path::Path;

pub use chrome::Chrome;
pub use http::Body;
pub use pages::{captured_page, HOME_ROUTE};
pub use preview::PreviewFiles;
pub use profiles::{Account, Profile};
pub use search::SEARCH_RESULTS_ALIAS;

use crate::{session::Session, Error};

pub struct Client {
    session: Session,
}

impl Client {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    pub fn open(session_path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self::new(Session::load(session_path)?))
    }

    pub fn session(&self) -> &Session {
        &self.session
    }
}
