use crate::cms::{Document, Filter, TileKind};

#[test]
fn resolves_standalone_collection() {
    let document = r#"{
      "data": {"type": "collection", "id": "c9",
               "attributes": {"title": "Later",
                              "component": {"id": "2-3", "mandatoryParams": "pf[x]=1", "customAttributes": {"grid": true},
                                            "filters": [{"id": "searchSuggestion",
                                                         "options": [{"value": "Crime", "parameter": "contentFilter[suggestion]=Crime"}]}]}},
               "relationships": {"items": {"data": [{"type": "collectionItem", "id": "ci1"}]}}},
      "included": [
        {"type": "collectionItem", "id": "ci1",
         "relationships": {"video": {"data": {"type": "video", "id": "v1"}}}},
        {"type": "video", "id": "v1", "attributes": {"name": "An Episode"}}
      ]
    }"#;
    let document = Document::parse(document).unwrap();
    assert!(document.page().is_none());
    let row = document.collection().unwrap();
    assert_eq!((row.id.as_str(), row.mandatory_parameters.as_deref()), ("c9", Some("pf[x]=1")));
    assert!(row.grid);
    assert_eq!(row.filters, [Filter { label: "Crime".into(), parameter: "contentFilter[suggestion]=Crime".into() }]);
    assert_eq!((row.tiles[0].kind, row.tiles[0].title.as_str()), (TileKind::Video, "An Episode"));
}

#[test]
fn resolves_tab_groups() {
    let document = r#"{
      "data": {"type": "collection", "id": "group", "attributes": {"component": {"id": "tab-group"}},
               "relationships": {"items": {"data": [{"type": "collectionItem", "id": "t1"},
                                                    {"type": "collectionItem", "id": "t2"}]}}},
      "included": [
        {"type": "collectionItem", "id": "t1", "relationships": {"collection": {"data": {"type": "collection", "id": "featured"}}}},
        {"type": "collectionItem", "id": "t2", "relationships": {"collection": {"data": {"type": "collection", "id": "action"}}}},
        {"type": "collection", "id": "featured", "attributes": {"title": "Featured", "component": {"id": "tab"}},
         "relationships": {"items": {"data": [{"type": "collectionItem", "id": "r1"}]}}},
        {"type": "collection", "id": "action", "attributes": {"title": "Action", "component": {"id": "tab"}},
         "relationships": {"items": {"data": [{"type": "collectionItem", "id": "r2"}]}}},
        {"type": "collectionItem", "id": "r1", "relationships": {"collection": {"data": {"type": "collection", "id": "rail1"}}}},
        {"type": "collectionItem", "id": "r2", "relationships": {"collection": {"data": {"type": "collection", "id": "rail2"}}}},
        {"type": "collection", "id": "rail1", "attributes": {"title": "Popular", "component": {"id": "2-3"}},
         "relationships": {"items": {"data": [{"type": "collectionItem", "id": "ci"}]}}},
        {"type": "collection", "id": "rail2", "attributes": {"title": "A-Z", "async": true, "component": {"id": "2-3"}}},
        {"type": "collectionItem", "id": "ci", "relationships": {"show": {"data": {"type": "show", "id": "s"}}}},
        {"type": "show", "id": "s", "attributes": {"name": "A Show"}}
      ]
    }"#;
    let group = Document::parse(document).unwrap().collection().unwrap();
    assert!(group.tiles.is_empty());
    let titles: Vec<_> = group.tabs.iter().map(|tab| tab.title.as_str()).collect();
    assert_eq!(titles, ["Featured", "Action"]);
    assert_eq!(group.tabs[0].rows[0].tiles[0].title, "A Show");
    let deferred = &group.tabs[1].rows[0];
    assert!(deferred.deferred && deferred.tiles.is_empty() && deferred.id == "rail2");
}
