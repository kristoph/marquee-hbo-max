use super::HOME_DOCUMENT;
use crate::cms::{Document, Layout, TileKind};

#[test]
fn resolves_page_rows_and_tiles() {
    let page = Document::parse(HOME_DOCUMENT).unwrap().page().unwrap();
    assert_eq!(page.title, "Home");
    assert_eq!(page.rows.len(), 2);

    let picks = &page.rows[0];
    assert_eq!((picks.title.as_str(), picks.component.as_str()), ("Picks", "2-3"));
    assert_eq!(picks.template.as_deref(), Some("primary"));
    assert!(!picks.deferred);
    assert_eq!(picks.tiles.len(), 2);

    let show = &picks.tiles[0];
    assert_eq!((show.kind, show.title.as_str()), (TileKind::Show, "Some Show"));
    assert_eq!(show.images.len(), 3);
    assert_eq!(picks.layout(), Layout::Poster);
    assert_eq!(show.artwork(Layout::Poster).unwrap().kind, "poster-with-logo");
    assert_eq!(show.artwork(Layout::Landscape).unwrap().kind, "default");
    assert_eq!(show.artwork(Layout::Square).unwrap().kind, "poster-with-logo");
    assert_eq!(show.route.as_deref(), Some("/video/watch/x"));

    let view = &picks.tiles[1];
    assert_eq!((view.kind, view.title.as_str()), (TileKind::View, "A View"));
    assert_eq!(view.route, None);
    assert!(view.artwork(Layout::Poster).is_none());

    assert_eq!(show.detail.rating.as_deref(), Some("TV-MA"));
    assert_eq!(show.detail.facts, ["TV-MA", "2 Seasons"]);
    let badge = show.detail.badge.as_ref().unwrap();
    assert_eq!((badge.label.as_str(), badge.background, badge.ink), ("New Episode", [186, 35, 108, 255], [255, 255, 255, 255]));
    assert_eq!(badge.icon.as_ref().unwrap().source, "https://img.example/bolt.png");
    assert!(show.detail.live);
    assert_eq!(show.detail.progress, Some(0.62));
    assert_eq!(show.detail.actions[0].label, "Resume");
    assert_eq!(show.detail.actions[0].route.as_deref(), Some("/video/watch/x"));
    assert_eq!(show.logo().unwrap().kind, "logo-centered");

    let later = &page.rows[1];
    assert!(later.deferred && later.tiles.is_empty());
}

#[test]
fn accepts_recorder_wrapper() {
    let wrapped = format!(r#"{{"capture": "routes-home", "status": 200, "body": {HOME_DOCUMENT}}}"#);
    assert_eq!(Document::parse(&wrapped).unwrap().page().unwrap().rows.len(), 2);
}
