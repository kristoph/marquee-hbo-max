use std::{collections::BTreeMap, time::Instant};

use eframe::egui::Rect;
use max_api::session::{Session, WebCookie, SIGN_IN_URL, USER_AGENT};
use wry::{WebView, WebViewBuilder};

use crate::{timing::COOKIE_STORE_SETTLING, web, Failure};

const SHOW_PAGE_AGAIN: &str = "window.__maxShowPage && window.__maxShowPage();";

pub struct WebPlayerRequest {
    pub home_url: String,
    pub headers: BTreeMap<String, String>,
}

impl WebPlayerRequest {
    fn from_message(message: &str) -> Option<Self> {
        let message: serde_json::Value = serde_json::from_str(message).ok()?;
        let headers = serde_json::from_value(message["headers"].clone()).ok()?;
        Some(Self { home_url: Session::home_url_in_region_of(message["api_origin"].as_str()?)?, headers })
    }
}

pub struct SignIn {
    view: WebView,
    bounds: Rect,
    load_at: Option<Instant>,
    pub request: Option<WebPlayerRequest>,
    pub checking: bool,
}

impl SignIn {
    pub fn embed(
        frame: &eframe::Frame,
        bounds: Rect,
        cookies_to_start_with: &[WebCookie],
        on_request: impl Fn(WebPlayerRequest) + 'static,
    ) -> Result<Self, Failure> {
        let view = WebViewBuilder::new()
            .with_user_agent(USER_AGENT)
            .with_bounds(web::bounds(bounds))
            .with_background_color(web::BLACK)
            .with_initialization_script(include_str!("signin.js"))
            .with_ipc_handler(move |request| {
                if let Some(request) = WebPlayerRequest::from_message(request.body()) {
                    on_request(request);
                }
            })
            .with_navigation_handler(|url| {
                println!("SIGN-IN {}", web::without_query(&url));
                true
            })
            .build_as_child(frame)?;
        for cookie in cookies_to_start_with {
            web::cookies::store(&view, cookie)?;
        }
        Ok(Self { view, bounds, load_at: Some(Instant::now() + COOKIE_STORE_SETTLING), request: None, checking: false })
    }

    pub fn is_waiting_to_load(&self) -> bool {
        self.load_at.is_some()
    }

    pub fn load_when_cookies_are_stored(&mut self) {
        if self.load_at.is_some_and(|due| Instant::now() >= due) {
            self.load_at = None;
            if let Err(error) = self.view.load_url(SIGN_IN_URL) {
                eprintln!("the sign-in page failed to load: {error}");
            }
            let _ = self.view.focus();
        }
    }

    pub fn keep_filling(&mut self, bounds: Rect) {
        if bounds != self.bounds {
            self.bounds = bounds;
            let _ = self.view.set_bounds(web::bounds(bounds));
        }
    }

    pub fn read_cookies(&self, deliver: impl Fn(Vec<WebCookie>) + 'static) {
        web::cookies::read_all(&self.view, deliver);
    }

    /// The web player's pages are hidden as they load; if the session they hold turns out not
    /// to be signed in after all, the person needs to see them.
    pub fn show_page_again(&mut self) {
        self.request = None;
        self.checking = false;
        let _ = self.view.evaluate_script(SHOW_PAGE_AGAIN);
    }
}
