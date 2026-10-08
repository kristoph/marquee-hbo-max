use crate::cms::{Document, NavigationItem};

#[test]
fn reads_navigation_items_and_named_images() {
    let document = r#"{
      "data": {"type": "collection", "id": "nav"},
      "included": [
        {"type": "collection", "id": "sec", "attributes": {"alias": "navigation-section-x-menu-categories"},
         "relationships": {"items": {"data": [{"type": "collectionItem", "id": "n1"},
                                              {"type": "collectionItem", "id": "n2"}]}}},
        {"type": "collectionItem", "id": "n1",
         "relationships": {"collection": {"data": {"type": "collection", "id": "series"}}}},
        {"type": "collectionItem", "id": "n2",
         "relationships": {"collection": {"data": {"type": "collection", "id": "hbo"}}}},
        {"type": "collection", "id": "series", "attributes": {"title": "Series"},
         "relationships": {"items": {"data": [{"type": "collectionItem", "id": "n3"}]}}},
        {"type": "collectionItem", "id": "n3", "relationships": {"link": {"data": {"type": "link", "id": "l1"}}}},
        {"type": "link", "id": "l1",
         "relationships": {"linkedContentRoutes": {"data": [{"type": "route", "id": "r1"}]}}},
        {"type": "route", "id": "r1", "attributes": {"url": "/series"}},
        {"type": "collection", "id": "hbo", "attributes": {"name": "wemo-nav-item-hbo"}},
        {"type": "image", "id": "i1",
         "attributes": {"name": "search-icon", "kind": "default", "src": "https://img.example/s.png", "width": 72, "height": 72}}
      ]
    }"#;
    let document = Document::parse(document).unwrap();
    assert_eq!(
        document.navigation_items(),
        [NavigationItem { label: "Series".into(), route: Some("/series".into()) }, NavigationItem { label: "HBO".into(), route: None },]
    );
    assert_eq!(document.image_named("search-icon").unwrap().source, "https://img.example/s.png");
    assert!(document.image_named("missing").is_none());
}
