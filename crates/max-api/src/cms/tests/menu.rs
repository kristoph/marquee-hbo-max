use crate::cms::{Document, MenuAction};

#[test]
fn builds_context_menus() {
    let document = r#"{
      "data": {"type": "collection", "id": "rail", "attributes": {"component": {"id": "16-9"}},
               "relationships": {"items": {"data": [{"type": "collectionItem", "id": "ci"}]}}},
      "included": [
        {"type": "collectionItem", "id": "ci",
         "relationships": {"video": {"data": {"type": "video", "id": "v"}},
                           "userActions": {"data": [{"type": "userAction", "id": "restart"},
                                                    {"type": "userAction", "id": "restart"},
                                                    {"type": "userAction", "id": "list"},
                                                    {"type": "userAction", "id": "rate"},
                                                    {"type": "userAction", "id": "drop"}]}}},
        {"type": "video", "id": "v", "attributes": {"name": "An Episode"}},
        {"type": "userAction", "id": "restart",
         "attributes": {"actionType": "navigate", "context": "restart",
                        "elements": {"label": {"label": "Restart"}, "routeParams": {"startPosition": 0}}},
         "relationships": {"route": {"data": {"type": "route", "id": "r"}}}},
        {"type": "route", "id": "r", "attributes": {"url": "/video/watch/v"}},
        {"type": "userAction", "id": "list",
         "attributes": {"actionType": "toggle", "context": "myList", "initialState": "on", "url": "/my-list/show/s"},
         "relationships": {"actionTemplate": {"data": {"type": "userActionTemplate", "id": "t-list"}}}},
        {"type": "userActionTemplate", "id": "t-list",
         "attributes": {"on": {"elements": {"label": {"label": "Add to My List"}, "feedback": {"label": "Added to My List"}}},
                        "off": {"elements": {"label": {"label": "Remove From My List"}, "feedback": {"label": "Removed From My List"}}}}},
        {"type": "userAction", "id": "rate",
         "attributes": {"actionType": "select", "context": "explicitSignals", "selected": "LIKE", "url": "/signals/show/s"},
         "relationships": {"actionTemplate": {"data": {"type": "userActionTemplate", "id": "t-rate"}}}},
        {"type": "userActionTemplate", "id": "t-rate",
         "attributes": {"elements": {"title": {"label": "Rate This Series"}},
                        "options": [{"value": "LIKE", "on": {"elements": {"label": {"label": "Like"}}},
                                     "off": {"elements": {"label": {"label": "Liked"}}}}]}},
        {"type": "userAction", "id": "drop",
         "attributes": {"actionType": "remove", "context": "continueWatching", "url": "/continue-watching/video/v"},
         "relationships": {"actionTemplate": {"data": {"type": "userActionTemplate", "id": "t-drop"}}}},
        {"type": "userActionTemplate", "id": "t-drop",
         "attributes": {"elements": {"label": {"label": "Remove From Continue Watching"}, "feedback": {"label": "Removed"}}}}
      ]
    }"#;
    let row = Document::parse(document).unwrap().collection().unwrap();
    let menu = &row.tiles[0].detail.menu;
    assert_eq!(menu.len(), 4, "the repeated restart action is listed once");
    assert_eq!(menu[0], MenuAction::Go { label: "Restart".into(), context: "restart".into(), route: "/video/watch/v?pos=0".into() });
    assert!(matches!(&menu[1], MenuAction::MyList { url, listed: false, add, removed, .. }
        if url == "/my-list/show/s" && add == "Add to My List" && removed == "Removed From My List"));
    assert!(matches!(&menu[2], MenuAction::Rate { title, options, chosen: Some(chosen), .. }
        if title == "Rate This Series" && options[0].chosen_label == "Liked" && chosen == "LIKE"));
    assert!(matches!(&menu[3], MenuAction::Remove { url, label, .. }
        if url == "/continue-watching/video/v" && label == "Remove From Continue Watching"));
}
