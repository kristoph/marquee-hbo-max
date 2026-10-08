use max_api::{
    client::PreviewFiles,
    cms::{Filter, NextVideo},
    playback::TitlePlayback,
    session::{Session, WebCookie},
    Error,
};

use crate::{
    hero::HeroTitle,
    model::{LoadedEpisodes, ScreenAccount, ScreenChrome, ScreenPage, ScreenRow, SearchQuery},
    signin::WebPlayerRequest,
};

pub struct SearchResults {
    pub row: ScreenRow,
    pub topics: Vec<Filter>,
}

pub enum Message {
    Page { route: String, remember: bool, page: Result<ScreenPage, Error> },
    Row(ScreenRow),
    Chrome(ScreenChrome),
    PlayRoute { title: String, route: Result<Option<String>, Error> },
    Preview { title: HeroTitle, files: Result<PreviewFiles, Error> },
    SearchResults { query: SearchQuery, results: Result<SearchResults, Error> },
    AccountChangeFailed(Error),
    PlayerLeft,
    Episodes { show_id: String, season_number: Option<u32>, loaded: Result<LoadedEpisodes, Error> },
    TitlePlayback { title: String, prepared: Result<TitlePlayback, Error> },
    NextVideo { video_id: String, next: Result<Option<NextVideo>, Error> },
    Account(Result<ScreenAccount, Error>),
    ProfileSwitched(Result<(), Error>),
    SignedOut,
    WebDataCleared,
    WebPlayerRequest(WebPlayerRequest),
    WebPlayerCookies(Vec<WebCookie>),
    SignInChecked(Result<Session, Error>),
}
