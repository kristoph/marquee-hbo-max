use super::{super::sections::sections_of, *};

fn watching() -> Watching {
    let video = PlayableVideo {
        video_id: "v1".to_string(),
        edit_id: "e1".to_string(),
        show_id: Some("s1".to_string()),
        video_type: "movie".to_string(),
        name: "A Film".to_string(),
        show_name: None,
        season_number: None,
        episode_number: None,
        resume_seconds: None,
    };
    let answer = json!({"videos": [
        {"type": "promo", "start": 0, "duration": 14.0},
        {"type": "main", "start": 14.0, "duration": 7760.5,
         "annotations": [{"type": "end-credits", "start": 7156.0, "end": 7200.0}, {"type": "end-credits", "start": 7291.5, "end": 7700.0}]}
    ]});
    let main = main_video(&answer).unwrap();
    Watching::of(video, &SessionIds { playback: "p".to_string(), application: "a".to_string() }, main, &sections_of(main)).unwrap()
}

#[test]
fn takes_the_runtime_and_credits_of_the_main_video() {
    let watching = watching();
    assert_eq!((watching.runtime_seconds, watching.credits_start_seconds), (7760.5, Some(7277.5)));
}

#[test]
fn a_marker_says_where_in_which_title_and_when() {
    assert_eq!(
        watching().marker(60.5, 1_791_475_368_171),
        json!({
            "editId": "e1", "positionSec": 60.5, "runtimeSec": 7760.5, "videoType": "movie", "playbackSessionId": "p",
            "playableStatus": "VOD", "creationTimeEpochMs": 1_791_475_368_171_u64, "programId": "v1", "showId": "s1",
            "creditsStartTimeSec": 7277.5, "mainContentDurationSec": 7760.5, "appSessionId": "a"
        })
    );
}
