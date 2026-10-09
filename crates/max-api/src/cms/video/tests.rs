use super::*;

const DOCUMENT: &str = r#"{
  "data": {"type": "route", "id": "r"},
  "included": [
    {"type": "video", "id": "v1",
     "attributes": {"videoType": "MOVIE", "viewingHistory": {"completed": false, "position": 84191, "viewed": true}},
     "relationships": {"edit": {"data": {"type": "edit", "id": "e1"}}, "show": {"data": {"type": "show", "id": "s1"}}}},
    {"type": "show", "id": "s2", "attributes": {"name": "A Series"}},
    {"type": "video", "id": "v2", "attributes": {"videoType": "EPISODE", "name": "Pilot", "seasonNumber": 3, "episodeNumber": 7, "viewingHistory": {"completed": true, "position": 5000}},
     "relationships": {"edit": {"data": {"type": "edit", "id": "e2"}}, "show": {"data": {"type": "show", "id": "s2"}}}},
    {"type": "video", "id": "v3", "attributes": {"videoType": "EXTRA"}}
  ]
}"#;

#[test]
fn reads_what_playing_a_video_needs_and_where_to_resume() {
    let document = Document::parse(DOCUMENT).unwrap();
    let movie = document.playable_video("v1").unwrap();
    assert_eq!((movie.edit_id.as_str(), movie.show_id.as_deref(), movie.video_type.as_str()), ("e1", Some("s1"), "movie"));
    assert_eq!(movie.resume_seconds, Some(84.191));
}

#[test]
fn a_finished_video_starts_over_and_one_without_an_edit_cannot_play() {
    let document = Document::parse(DOCUMENT).unwrap();
    let episode = document.playable_video("v2").unwrap();
    assert_eq!((episode.resume_seconds, episode.season_number, episode.episode_number), (None, Some(3), Some(7)));
    assert_eq!((episode.name.as_str(), episode.show_name.as_deref()), ("Pilot", Some("A Series")));
    assert_eq!(document.playable_video("v3"), None);
}

#[test]
fn reads_the_video_that_follows_from_the_root_of_its_document() {
    let document =
        Document::parse(r#"{"data": {"type": "video", "id": "v9", "attributes": {"name": "Wagon Burner", "seasonNumber": 1, "episodeNumber": 2}}}"#)
            .unwrap();
    let next = document.next_video().unwrap();
    assert_eq!((next.route.as_str(), next.name.as_str(), next.season_and_episode), ("/video/watch/v9", "Wagon Burner", Some((1, 2))));
    assert_eq!(Document::parse(DOCUMENT).unwrap().next_video(), None);
}
