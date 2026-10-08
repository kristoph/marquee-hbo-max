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
mod tests {
    use max_api::cms::TileDetail;

    use super::*;

    fn tile(menu: Vec<MenuAction>, progress: Option<f32>) -> ScreenTile {
        let detail = TileDetail { menu, progress, ..TileDetail::default() };
        ScreenTile { title: String::new(), route: None, artwork: None, logo: None, badge_icon: None, banner_icon: None, detail }
    }

    fn go(label: &str, context: &str) -> MenuAction {
        MenuAction::Go { label: label.to_string(), context: context.to_string(), route: format!("/{context}") }
    }

    fn rating(value: &str) -> RatingOption {
        RatingOption { value: value.to_string(), label: value.to_lowercase(), chosen_label: format!("{}d", value.to_lowercase()) }
    }

    #[test]
    fn resuming_plays_and_shows_how_much_was_watched_while_more_info_opens_a_page() {
        let tile = tile(vec![go("Resume", "resume"), go("Restart", "restart"), go("More Info", "generic")], Some(0.4));
        let lines = menu_lines(&tile);
        assert!(lines[0].pick == Pick::Play("/resume") && lines[0].watched == Some(0.4));
        assert!(lines[1].pick == Pick::Play("/restart") && lines[1].watched.is_none() && lines[1].icon == icon::RESTART);
        assert!(lines[2].pick == Pick::OpenPage("/generic") && lines[2].icon == icon::INFO);
    }

    #[test]
    fn my_list_offers_the_opposite_of_its_state_and_ratings_follow_under_a_rule() {
        let my_list = MenuAction::MyList {
            url: String::new(),
            listed: true,
            add: "Add".to_string(),
            remove: "Remove".to_string(),
            added: String::new(),
            removed: String::new(),
        };
        let rate = MenuAction::Rate {
            url: String::new(),
            title: String::new(),
            options: vec![rating("LOVE"), rating("LIKE"), rating("DISLIKE")],
            chosen: Some("LIKE".to_string()),
        };
        let tile = tile(vec![my_list, rate], None);
        let lines = menu_lines(&tile);
        assert_eq!(lines.iter().map(|line| line.label).collect::<Vec<_>>(), ["Remove", "love", "liked", "dislike"]);
        assert_eq!(lines.iter().map(|line| line.ruled_above).collect::<Vec<_>>(), [false, true, false, false]);
        assert_eq!(
            lines.iter().map(|line| line.icon).collect::<Vec<_>>(),
            [icon::CHECK, icon::HEART_OUTLINE, icon::THUMBS_UP, icon::THUMBS_DOWN_OUTLINE]
        );
        assert!(lines[2].chosen && !lines[1].chosen);
    }
}
