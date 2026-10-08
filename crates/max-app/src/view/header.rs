use eframe::egui::{self, Align2, Color32, Id, Image, Painter, Pos2, Rect, Response, Sense, Stroke, Vec2};
use max_api::client::HOME_ROUTE;

use crate::{
    app::App,
    intent::{Go, Intent, ProfileRequest},
    metrics::{
        header::{AVATAR, HEIGHT, ICON, LOGO},
        MARGIN,
    },
    paint::{gradient, points_to_when_hovered, CornerColors},
    routes,
    theme::{bold, BACKGROUND, TEXT},
};

const SHADE_BELOW_BAR: f32 = 40.0;
const SHADE_OVER_HERO: u8 = 170;
const HOVER_HALO: f32 = 10.0;
const HOVER_FILL: u8 = 28;
const LOGO_AFTER_MENU_BUTTON: f32 = 16.0;
const MY_STUFF_BEFORE_AVATAR: f32 = 22.0;
const SEARCH_BEFORE_MY_STUFF: f32 = 24.0;
const MENU_BAR_OFFSETS: [f32; 3] = [-0.28, 0.0, 0.28];
const FALLBACK_LOGO_TEXT: f32 = 20.0;
const FALLBACK_MY_STUFF_TEXT: f32 = 24.0;

#[derive(Clone, Copy)]
struct HeaderButton<'image> {
    name: &'static str,
    center: Pos2,
    size: f32,
    image: Option<&'image String>,
    round: bool,
}

impl App {
    pub(crate) fn draw_header(&self, ui: &mut egui::Ui, window: Rect, solid: bool, intent: &mut Intent) {
        paint_bar(ui.painter(), window, solid);
        let middle = window.top() + HEIGHT / 2.0;

        let menu_center = Pos2::new(window.left() + MARGIN + ICON / 2.0, middle);
        let menu = HeaderButton { name: "menu", center: menu_center, size: ICON, image: None, round: false };
        if draw_button(ui, menu, paint_menu_bars).clicked() {
            intent.browse_menu = Some(!self.browse_menu_open);
        }
        let logo = Rect::from_min_size(Pos2::new(menu_center.x + ICON / 2.0 + LOGO_AFTER_MENU_BUTTON, middle - LOGO.y / 2.0), LOGO);
        if self.draw_logo(ui, logo).clicked() {
            intent.go = Some(Go::Page(HOME_ROUTE.to_string()));
        }

        let avatar_center = Pos2::new(window.right() - MARGIN - AVATAR / 2.0, middle);
        let avatar = HeaderButton { name: "profile", center: avatar_center, size: AVATAR, image: self.chrome.avatar.as_ref(), round: true };
        let avatar_fallback = |painter: &Painter, area: Rect| {
            painter.circle_filled(area.center(), area.width() / 2.0, Color32::from_white_alpha(40));
        };
        if draw_button(ui, avatar, avatar_fallback).clicked() {
            intent.profile = Some(ProfileRequest::OpenPicker);
        }

        let my_stuff_center = Pos2::new(avatar_center.x - AVATAR / 2.0 - MY_STUFF_BEFORE_AVATAR - ICON / 2.0, middle);
        let my_stuff =
            HeaderButton { name: "my-stuff", center: my_stuff_center, size: ICON, image: self.chrome.my_stuff_icon.as_ref(), round: false };
        let my_stuff_fallback = |painter: &Painter, area: Rect| {
            painter.text(area.center(), Align2::CENTER_CENTER, "+", bold(FALLBACK_MY_STUFF_TEXT), TEXT);
        };
        if draw_button(ui, my_stuff, my_stuff_fallback).clicked() {
            intent.go = Some(Go::Page(routes::MY_STUFF.to_string()));
        }

        let search_center = Pos2::new(my_stuff_center.x - ICON - SEARCH_BEFORE_MY_STUFF, middle);
        let search = HeaderButton { name: "search", center: search_center, size: ICON, image: self.chrome.search_icon.as_ref(), round: false };
        let search_fallback = |painter: &Painter, area: Rect| {
            painter.circle_stroke(area.center(), area.width() * 0.3, Stroke::new(2.0, TEXT));
        };
        if draw_button(ui, search, search_fallback).clicked() {
            intent.go = Some(Go::Page(routes::SEARCH.to_string()));
        }
    }

    fn draw_logo(&self, ui: &mut egui::Ui, logo: Rect) -> Response {
        let response = ui.interact(logo.expand(6.0), Id::new(("header", "logo")), Sense::click());
        points_to_when_hovered(ui, &response);
        match &self.chrome.logo {
            Some(uri) => {
                Image::new(uri.as_str()).paint_at(ui, logo);
            }
            None => {
                ui.painter().text(logo.left_center(), Align2::LEFT_CENTER, "HBO Max", bold(FALLBACK_LOGO_TEXT), TEXT);
            }
        }
        response
    }
}

fn paint_bar(painter: &Painter, window: Rect, solid: bool) {
    let clear = Color32::TRANSPARENT;
    let bar = Rect::from_min_size(window.min, Vec2::new(window.width(), HEIGHT));
    let below = Rect::from_min_size(bar.left_bottom(), Vec2::new(window.width(), SHADE_BELOW_BAR));
    if solid {
        painter.rect_filled(bar, 0.0, BACKGROUND);
        gradient(painter, below, CornerColors::top_to_bottom(BACKGROUND, clear));
    } else {
        gradient(painter, bar.union(below), CornerColors::top_to_bottom(Color32::from_black_alpha(SHADE_OVER_HERO), clear));
    }
}

fn paint_menu_bars(painter: &Painter, area: Rect) {
    for offset in MENU_BAR_OFFSETS {
        let height = area.center().y + area.height() * offset;
        painter.line_segment([Pos2::new(area.left() + 3.0, height), Pos2::new(area.right() - 3.0, height)], Stroke::new(2.0, TEXT));
    }
}

fn draw_button(ui: &mut egui::Ui, button: HeaderButton, paint_fallback: impl FnOnce(&Painter, Rect)) -> Response {
    let area = Rect::from_center_size(button.center, Vec2::splat(button.size));
    let response = ui.interact(area.expand(HOVER_HALO), Id::new(("header", button.name)), Sense::click());
    if response.hovered() {
        let radius = if button.round { 99.0 } else { 6.0 };
        ui.painter().rect_filled(area.expand(HOVER_HALO), radius, Color32::from_white_alpha(HOVER_FILL));
    }
    points_to_when_hovered(ui, &response);
    match button.image {
        Some(uri) => {
            Image::new(uri.as_str()).corner_radius(if button.round { button.size / 2.0 } else { 0.0 }).paint_at(ui, area);
        }
        None => paint_fallback(ui.painter(), area),
    }
    response
}
