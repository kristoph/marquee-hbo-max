use std::time::Instant;

use eframe::egui::Rect;
use max_api::session::{Session, USER_AGENT, WEB_ORIGIN};
use wry::{WebView, WebViewBuilder};

use crate::{timing::COOKIE_STORE_SETTLING, web, Failure};

const CLOSE_MESSAGE: &str = "close";
/// Names a script to run in the player as well, for studying what the web player does.
const EXTRA_SCRIPT_VARIABLE: &str = "MAX_PLAYER_SCRIPT";

#[derive(Clone, Copy)]
pub struct PlayerOptions<'session> {
    pub session: &'session Session,
    pub route: &'session str,
    pub bounds: Rect,
    pub muted: bool,
}

pub struct Player {
    view: WebView,
    bounds: Rect,
    pending_load: Option<(String, Instant)>,
}

impl Player {
    pub fn embed(frame: &eframe::Frame, options: PlayerOptions, on_close: impl Fn() + 'static) -> Result<Self, Failure> {
        if options.session.web_cookies.is_empty() {
            return Err("the saved session has no web cookies; sign in again".into());
        }
        let extra_script = std::env::var(EXTRA_SCRIPT_VARIABLE).ok().and_then(|path| std::fs::read_to_string(path).ok());
        let view = WebViewBuilder::new()
            .with_user_agent(USER_AGENT)
            .with_bounds(web::bounds(options.bounds))
            .with_background_color(web::BLACK)
            .with_initialization_script(format!("window.__maxrsMuted = {};", options.muted))
            .with_initialization_script(include_str!("player.js"))
            .with_initialization_script(extra_script.unwrap_or_default())
            .with_ipc_handler(move |request| match request.body().as_str() {
                CLOSE_MESSAGE => on_close(),
                message => println!("PLAYER SAYS {message}"),
            })
            .with_navigation_handler(|url| {
                println!("PLAYER {}", web::without_query(&url));
                true
            })
            .build_as_child(frame)?;
        for cookie in &options.session.web_cookies {
            web::cookies::store(&view, cookie)?;
        }
        let load_at = Instant::now() + COOKIE_STORE_SETTLING;
        Ok(Self { view, bounds: options.bounds, pending_load: Some((format!("{WEB_ORIGIN}{}", options.route), load_at)) })
    }

    pub fn is_waiting_to_load(&self) -> bool {
        self.pending_load.is_some()
    }

    pub fn keep_filling(&mut self, bounds: Rect) {
        if bounds != self.bounds {
            self.bounds = bounds;
            let _ = self.view.set_bounds(web::bounds(bounds));
        }
    }

    pub fn load_when_signed_in(&mut self) {
        let Some((url, due)) = self.pending_load.take() else { return };
        if Instant::now() < due {
            self.pending_load = Some((url, due));
            return;
        }
        if let Err(error) = self.view.load_url(&url) {
            eprintln!("player failed to load: {error}");
        }
        let _ = self.view.focus();
    }
}
