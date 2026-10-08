use max_api::cms::{Action, Layout, MenuAction, TileDetail};

use super::lazy_image::{LazyImage, SizedImage};

pub struct ScreenTile {
    pub title: String,
    pub route: Option<String>,
    pub artwork: Option<LazyImage>,
    pub logo: Option<SizedImage>,
    pub badge_icon: Option<SizedImage>,
    pub banner_icon: Option<SizedImage>,
    pub detail: TileDetail,
}

impl ScreenTile {
    pub fn play_action(&self) -> Option<&Action> {
        self.detail.actions.iter().find(|action| action.plays())
    }

    pub fn is_on_my_list(&self) -> Option<bool> {
        self.detail.menu.iter().find_map(|entry| match entry {
            MenuAction::MyList { listed, .. } => Some(*listed),
            _ => None,
        })
    }
}

pub struct RankNumeral {
    pub plain: Option<LazyImage>,
    pub selected: Option<LazyImage>,
}

pub struct ScreenRow {
    pub id: String,
    pub title: String,
    pub layout: Layout,
    pub numbered: bool,
    pub masthead: Option<SizedImage>,
    pub rank_numerals: Vec<RankNumeral>,
    pub tiles: Vec<ScreenTile>,
    pub pending: bool,
    pub grid_line: bool,
    pub resumes_watching: bool,
}

impl ScreenRow {
    pub fn untitled(id: String, layout: Layout, tiles: Vec<ScreenTile>) -> Self {
        Self {
            id,
            title: String::new(),
            layout,
            numbered: false,
            masthead: None,
            rank_numerals: Vec::new(),
            tiles,
            pending: false,
            grid_line: false,
            resumes_watching: false,
        }
    }

    pub fn takes_space(&self) -> bool {
        self.pending || !self.tiles.is_empty()
    }

    pub fn is_hero(&self) -> bool {
        self.layout == Layout::Hero && !self.tiles.is_empty()
    }
}
