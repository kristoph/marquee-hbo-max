mod account;
mod episodes;
mod fade;
mod failures;
mod frame;
mod keys;
mod messages;
mod native_playback;
mod navigation;
mod playback;
mod profiles;
mod search;
mod selection;
mod signin;
mod tabs;
mod title_controls;

use std::{cell::RefCell, collections::HashMap, ffi::c_void, path::PathBuf, ptr::NonNull, sync::mpsc, thread, time::Instant};

use eframe::egui;
use max_media::images::OpaqueBounds;
pub use navigation::Navigation;

use crate::{
    developer::DeveloperTools,
    hero::HeroState,
    intent::TileMenu,
    message::Message,
    model::{load_chrome, OpenPage, ProfilePicker, ScreenChrome},
    player::Playback,
    service::Service,
    signin::SignIn,
    start::Start,
    theme,
    transition::{Fade, FadePhase},
};

pub struct Toast {
    pub message: String,
    pub shown_at: Instant,
}

pub struct App {
    pub(crate) service: Service,
    pub(crate) session_path: PathBuf,
    pub(crate) sender: mpsc::Sender<Message>,
    pub(crate) inbox: mpsc::Receiver<Message>,
    pub(crate) chrome: ScreenChrome,
    pub(crate) page: OpenPage,
    pub(crate) navigation: Navigation,
    pub(crate) hero: HeroState,
    pub(crate) browse_menu_open: bool,
    pub(crate) tile_menu: Option<TileMenu>,
    pub(crate) toast: Option<Toast>,
    pub(crate) profile_picker: Option<ProfilePicker>,
    pub(crate) sign_in: Option<SignIn>,
    pub(crate) sign_in_wanted: bool,
    pub(crate) playback: Playback,
    pub(crate) window_view: Option<NonNull<c_void>>,
    pub(crate) fade: Option<Fade>,
    pub(crate) logo_bounds: RefCell<HashMap<String, OpaqueBounds>>,
    pub(crate) developer: DeveloperTools,
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext<'_>, start: Start) -> Self {
        let ctx = &creation_context.egui_ctx;
        egui_extras::install_image_loaders(ctx);
        theme::install_fonts(ctx);
        ctx.set_visuals(egui::Visuals::dark());
        if start.background_position.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        let (sender, inbox) = mpsc::channel();
        let hero_title = if start.selected.row == 0 { start.selected.column } else { 0 };
        let mut app = Self {
            service: start.service,
            session_path: start.session_path,
            sender,
            inbox,
            chrome: ScreenChrome::default(),
            page: OpenPage::empty(start.selected),
            navigation: Navigation::starting_at(start.route.clone()),
            hero: HeroState::showing(hero_title),
            browse_menu_open: start.browse_menu_open,
            tile_menu: None,
            toast: None,
            profile_picker: None,
            sign_in: None,
            sign_in_wanted: false,
            playback: Playback::new(start.player_choice, start.mute_player),
            window_view: None,
            fade: Some(Fade::starting(FadePhase::Black)),
            logo_bounds: RefCell::new(HashMap::new()),
            developer: DeveloperTools::new(start.developer),
        };
        if matches!(app.service, Service::SignedOut) {
            match app.developer.clear_web_data {
                true => app.clear_web_data(ctx),
                false => app.sign_in_wanted = true,
            }
            return app;
        }
        app.start_loading(ctx, start.route, false);
        if let Some(route) = start.play {
            app.start_playing(ctx, route.clone(), route);
        }
        app.load_chrome(ctx);
        app
    }

    pub(crate) fn load_chrome(&self, ctx: &egui::Context) {
        let (sender, ctx, service) = (self.sender.clone(), ctx.clone(), self.service.clone());
        thread::spawn(move || {
            match load_chrome(&service) {
                Ok(chrome) => drop(sender.send(Message::Chrome(chrome))),
                Err(error) => log::warn!("header failed: {error}"),
            }
            ctx.request_repaint();
        });
    }

    /// Leaves nothing of what was being browsed, as when the account or its profile changes.
    pub(crate) fn clear_browsing(&mut self) {
        self.page.clear();
        self.navigation.history.clear();
        self.navigation.visited.clear();
        self.browse_menu_open = false;
        self.tile_menu = None;
        self.profile_picker = None;
        self.playback.close();
        self.hero.preview = None;
    }

    pub(crate) fn start_afresh(&mut self, ctx: &egui::Context, route: &str) {
        self.clear_browsing();
        self.fade = Some(Fade::starting(FadePhase::Black));
        self.start_loading(ctx, route.to_string(), false);
    }

    pub(crate) fn something_covers_the_page(&self) -> bool {
        self.playback.is_open() || self.sign_in.is_some()
    }

    pub(crate) fn say(&mut self, message: String) {
        self.toast = Some(Toast { message, shown_at: Instant::now() });
    }
}
