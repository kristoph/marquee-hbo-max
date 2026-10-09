use super::*;

fn detail_page(listed_actions: &[&str]) -> Document {
    let listed: Vec<String> = listed_actions.iter().map(|id| format!(r#"{{"type": "userAction", "id": "{id}"}}"#)).collect();
    Document::parse(&format!(
        r#"{{
          "data": {{"type": "route", "id": "r", "relationships": {{"target": {{"data": {{"type": "page", "id": "p"}}}}}}}},
          "included": [
            {{"type": "page", "id": "p", "relationships": {{"items": {{"data": [{{"type": "pageItem", "id": "pi"}}]}}}}}},
            {{"type": "pageItem", "id": "pi", "relationships": {{"collection": {{"data": {{"type": "collection", "id": "c"}}}}}}}},
            {{"type": "collection", "id": "c", "attributes": {{"component": {{"id": "hero"}}}},
              "relationships": {{"items": {{"data": [{{"type": "collectionItem", "id": "ci"}}]}}}}}},
            {{"type": "collectionItem", "id": "ci",
              "relationships": {{"show": {{"data": {{"type": "show", "id": "s"}}}}, "userActions": {{"data": [{}]}}}}}},
            {{"type": "show", "id": "s", "attributes": {{"name": "A Show"}}}},
            {{"type": "userAction", "id": "info", "attributes": {{"context": "generic", "elements": {{"label": {{"label": "More Info"}}}}}},
              "relationships": {{"route": {{"data": {{"type": "route", "id": "r-info"}}}}}}}},
            {{"type": "userAction", "id": "resume", "attributes": {{"context": "resume", "elements": {{"label": {{"label": "Resume S1 E3"}}}}}},
              "relationships": {{"route": {{"data": {{"type": "route", "id": "r-ep"}}}}}}}},
            {{"type": "userAction", "id": "trailer", "attributes": {{"context": "play", "elements": {{"label": {{"label": "Trailer"}}}}}},
              "relationships": {{"route": {{"data": {{"type": "route", "id": "r-trailer"}}}}}}}},
            {{"type": "route", "id": "r-info", "attributes": {{"url": "/show/s"}}}},
            {{"type": "route", "id": "r-ep", "attributes": {{"url": "/video/watch/ep3"}}}},
            {{"type": "route", "id": "r-trailer", "attributes": {{"url": "/video/watch/trailer"}}}}
          ]
        }}"#,
        listed.join(",")
    ))
    .unwrap()
}

#[test]
fn the_episode_to_resume_wins_over_the_trailer_listed_after_it() {
    assert_eq!(play_route_of(&detail_page(&["info", "resume", "trailer"])).as_deref(), Some("/video/watch/ep3"));
}

#[test]
fn a_page_with_nothing_to_play_has_no_play_route() {
    assert_eq!(play_route_of(&detail_page(&["info"])), None);
}
