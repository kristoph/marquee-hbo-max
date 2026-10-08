use eframe::egui::{Pos2, Rect, Vec2};

use crate::{metrics::rail::MORE_BUTTON, model::Selection, routes};

pub enum Go {
    Page(String),
    Play { route: String, title: String },
    Back,
}

impl Go {
    pub fn to(route: &str, title: &str) -> Self {
        match routes::leads_to_a_title(route) {
            true => Go::Play { route: route.to_string(), title: title.to_string() },
            false => Go::Page(route.to_string()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TileMenu {
    pub tile: Selection,
    pub position: Pos2,
    pub cursor: Option<usize>,
}

impl TileMenu {
    pub fn at(tile: Selection, position: Pos2) -> Self {
        Self { tile, position, cursor: None }
    }

    pub fn beside_artwork(tile: Selection, artwork: Rect) -> Self {
        Self::at(tile, artwork.right_top() + Vec2::new(-MORE_BUTTON, MORE_BUTTON))
    }
}

pub enum AccountChangeKind {
    ToggleMyList,
    Rate(String),
    RemoveFromRow,
}

pub struct AccountChange {
    pub tile: Selection,
    pub kind: AccountChangeKind,
}

#[derive(Clone, Copy)]
pub enum ProfileRequest {
    OpenPicker,
    ClosePicker,
    Choose(usize),
    SubmitPin,
    CancelPin,
    SignOut,
}

pub enum TileMenuRequest {
    Open(TileMenu),
    Close,
}

#[derive(Default)]
pub struct Intent {
    pub go: Option<Go>,
    pub select: Option<Selection>,
    pub browse_menu: Option<bool>,
    pub tab: Option<usize>,
    pub toggle_preview_sound: bool,
    pub hero_drag: f32,
    pub hero_drag_ended: bool,
    pub hero_sideways_scrolls: Vec<f32>,
    pub tile_menu: Option<TileMenuRequest>,
    pub account_change: Option<AccountChange>,
    pub profile: Option<ProfileRequest>,
}
