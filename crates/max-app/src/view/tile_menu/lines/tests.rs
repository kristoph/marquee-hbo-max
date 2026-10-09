use max_api::cms::TileDetail;

use super::*;

fn tile(menu: Vec<MenuAction>, progress: Option<f32>) -> ScreenTile {
    ScreenTile::with(TileDetail { menu, progress, ..TileDetail::default() })
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
