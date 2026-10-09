use super::*;

fn section(kind: SectionKind, label: &str, start_seconds: f64, end_seconds: f64) -> Section {
    Section { kind, label: label.to_string(), start_seconds, end_seconds }
}

fn next() -> NextVideo {
    NextVideo { route: "/video/watch/next".to_string(), name: "Next".to_string(), season_and_episode: Some((1, 2)) }
}

fn prompt_at(position_seconds: f64, sections: &[Section], next: Option<&NextVideo>, watching_credits: bool) -> Option<String> {
    let moment = Moment { position_seconds, duration_seconds: 1000.0, sections, next, watching_credits };
    moment.prompt().map(|prompt| match prompt {
        Prompt::Skip { label, to_seconds } => format!("{label} to {to_seconds}"),
        Prompt::UpNext { counted_down, .. } => format!("next {counted_down:.1}"),
    })
}

#[test]
fn offers_to_skip_only_while_the_section_plays() {
    let sections = [section(SectionKind::Skippable, "Skip Intro", 5.0, 80.0)];
    assert_eq!(prompt_at(4.0, &sections, None, false), None);
    assert_eq!(prompt_at(30.0, &sections, None, false).as_deref(), Some("Skip Intro to 80"));
    assert_eq!(prompt_at(80.0, &sections, None, false), None);
}

#[test]
fn counts_down_to_the_next_episode_from_the_start_of_the_credits() {
    let sections = [section(SectionKind::Credits, "Next Episode", 900.0, 960.0)];
    let next = next();
    assert_eq!(prompt_at(899.0, &sections, Some(&next), false), None);
    assert_eq!(prompt_at(907.5, &sections, Some(&next), false).as_deref(), Some("next 0.5"));
    assert_eq!(prompt_at(990.0, &sections, Some(&next), false).as_deref(), Some("next 1.0"));
}

#[test]
fn offers_nothing_without_a_next_episode_or_once_the_credits_were_chosen() {
    let sections = [section(SectionKind::Credits, "Next Episode", 900.0, 960.0)];
    assert_eq!(prompt_at(950.0, &sections, None, false), None);
    assert_eq!(prompt_at(950.0, &sections, Some(&next()), true), None);
}

#[test]
fn without_marked_credits_the_last_seconds_offer_the_next_episode() {
    let next = next();
    assert_eq!(prompt_at(980.0, &[], Some(&next), false), None);
    assert_eq!(prompt_at(992.5, &[], Some(&next), false).as_deref(), Some("next 0.5"));
}
