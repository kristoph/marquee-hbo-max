use crate::cms::{Document, Layout, TileKind};

#[test]
fn resolves_channel_tiles() {
    let document = r#"{
      "data": {"type": "collection", "id": "channels",
               "attributes": {"title": "Channels", "component": {"id": "multilevel"}},
               "relationships": {"items": {"data": [{"type": "collectionItem", "id": "ci1"}]}}},
      "included": [
        {"type": "collectionItem", "id": "ci1",
         "relationships": {"collection": {"data": {"type": "collection", "id": "nested"}}}},
        {"type": "collection", "id": "nested", "attributes": {"name": "virtual-channels-horror"},
         "relationships": {"items": {"data": [{"type": "collectionItem", "id": "ci2"}]}}},
        {"type": "collectionItem", "id": "ci2",
         "relationships": {"airing": {"data": {"type": "airing", "id": "air"}},
                           "defaultAction": {"data": {"type": "userAction", "id": "act"}}}},
        {"type": "airing", "id": "air",
         "attributes": {"name": "Mockingbird", "showName": "Game of Thrones"},
         "relationships": {"distributionChannel": {"data": {"type": "distributionChannel", "id": "dc"}}}},
        {"type": "distributionChannel", "id": "dc", "attributes": {"name": "World of Westeros"},
         "relationships": {"images": {"data": [{"type": "image", "id": "i1"}]}}},
        {"type": "image", "id": "i1",
         "attributes": {"kind": "cover-artwork-horizontal", "src": "https://img.example/w.jpeg",
                        "width": 3840, "height": 2160}},
        {"type": "userAction", "id": "act",
         "attributes": {"context": "play", "elements": {"label": {"label": "Watch Channel"}}},
         "relationships": {"route": {"data": {"type": "route", "id": "r1"}}}},
        {"type": "route", "id": "r1", "attributes": {"url": "/channel/watch/x"}}
      ]
    }"#;
    let row = Document::parse(document).unwrap().collection().unwrap();
    assert_eq!(row.layout(), Layout::Landscape);
    let tile = &row.tiles[0];
    assert_eq!((tile.kind, tile.title.as_str()), (TileKind::Channel, "World of Westeros"));
    assert_eq!(tile.detail.secondary_title.as_deref(), Some("Game of Thrones: Mockingbird"));
    assert_eq!(tile.route.as_deref(), Some("/channel/watch/x"));
    assert_eq!(tile.artwork(row.layout()).unwrap().kind, "cover-artwork-horizontal");
}
