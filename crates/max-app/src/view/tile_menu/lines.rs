use max_api::cms::{MenuAction, RatingOption};

use crate::{model::ScreenTile, theme::icon};

const CONTEXTS_THAT_OPEN_A_PAGE: [&str; 2] = ["generic", "moreInfo"];
const RESUME_CONTEXT: &str = "resume";
const RESTART_CONTEXT: &str = "restart";

#[derive(Clone, Copy, PartialEq)]
pub enum Pick<'tile> {
    OpenPage(&'tile str),
    Play(&'tile str),
    ToggleMyList,
    RemoveFromRow,
    Rate(&'tile str),
}

pub struct MenuLine<'tile> {
    pub label: &'tile str,
    pub pick: Pick<'tile>,
    pub icon: char,
    pub chosen: bool,
    pub ruled_above: bool,
    pub watched: Option<f32>,
}

impl<'tile> MenuLine<'tile> {
    fn plain(label: &'tile str, pick: Pick<'tile>, icon: char) -> Self {
        Self { label, pick, icon, chosen: false, ruled_above: false, watched: None }
    }
}

pub fn menu_lines(tile: &ScreenTile) -> Vec<MenuLine<'_>> {
    let mut lines = Vec::new();
    for entry in &tile.detail.menu {
        match entry {
            MenuAction::Go { label, context, route } => {
                let opens_a_page = CONTEXTS_THAT_OPEN_A_PAGE.contains(&context.as_str());
                let (pick, icon) = match context.as_str() {
                    _ if opens_a_page => (Pick::OpenPage(route), icon::INFO),
                    RESTART_CONTEXT => (Pick::Play(route), icon::RESTART),
                    _ => (Pick::Play(route), icon::PLAY),
                };
                let watched = tile.detail.progress.filter(|_| context == RESUME_CONTEXT);
                lines.push(MenuLine { watched, ..MenuLine::plain(label, pick, icon) });
            }
            MenuAction::MyList { listed: true, remove, .. } => lines.push(MenuLine::plain(remove, Pick::ToggleMyList, icon::CHECK)),
            MenuAction::MyList { listed: false, add, .. } => lines.push(MenuLine::plain(add, Pick::ToggleMyList, icon::PLUS)),
            MenuAction::Remove { label, .. } => lines.push(MenuLine::plain(label, Pick::RemoveFromRow, icon::REMOVE)),
            MenuAction::Rate { options, chosen, .. } => {
                let first_rating = lines.len();
                lines.extend(options.iter().map(|option| rating_line(option, chosen.as_ref() == Some(&option.value))));
                if let Some(first) = lines.get_mut(first_rating).filter(|_| first_rating > 0) {
                    first.ruled_above = true;
                }
            }
        }
    }
    lines
}

fn rating_line(option: &RatingOption, chosen: bool) -> MenuLine<'_> {
    let value = option.value.as_str();
    let icon = match (value, chosen) {
        _ if value.ends_with("LOVE") => {
            if chosen {
                icon::HEART
            } else {
                icon::HEART_OUTLINE
            }
        }
        _ if value.ends_with("LIKE") && !value.ends_with("DISLIKE") => {
            if chosen {
                icon::THUMBS_UP
            } else {
                icon::THUMBS_UP_OUTLINE
            }
        }
        (_, true) => icon::THUMBS_DOWN,
        (_, false) => icon::THUMBS_DOWN_OUTLINE,
    };
    let label = if chosen { &option.chosen_label } else { &option.label };
    MenuLine { chosen, ..MenuLine::plain(label, Pick::Rate(value), icon) }
}

#[cfg(test)]
mod tests;
