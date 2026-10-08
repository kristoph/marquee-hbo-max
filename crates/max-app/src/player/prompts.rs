//! What the player offers at particular moments of a title: skipping an opening or a recap,
//! and going on to the next episode once the credits roll.

use max_api::{
    cms::NextVideo,
    playback::{Section, SectionKind},
};

/// How long the next episode waits before starting by itself, as in the service's own apps.
const UP_NEXT_COUNTDOWN_SECONDS: f64 = 15.0;

#[derive(Debug, PartialEq)]
pub enum Prompt<'title> {
    Skip { label: &'title str, to_seconds: f64 },
    UpNext { next: &'title NextVideo, counted_down: f32 },
}

pub struct Moment<'title> {
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub sections: &'title [Section],
    pub next: Option<&'title NextVideo>,
    pub watching_credits: bool,
}

impl<'title> Moment<'title> {
    pub fn prompt(&self) -> Option<Prompt<'title>> {
        self.skip().or_else(|| self.up_next())
    }

    fn skip(&self) -> Option<Prompt<'title>> {
        let section = self.sections.iter().find(|section| section.kind == SectionKind::Skippable && section.contains(self.position_seconds))?;
        Some(Prompt::Skip { label: &section.label, to_seconds: section.end_seconds })
    }

    /// A title whose credits the service has not marked offers the next one over its last seconds.
    fn up_next(&self) -> Option<Prompt<'title>> {
        let next = self.next.filter(|_| !self.watching_credits)?;
        let credits = self.sections.iter().rfind(|section| section.kind == SectionKind::Credits);
        let offered_from = credits.map_or(self.duration_seconds - UP_NEXT_COUNTDOWN_SECONDS, |credits| credits.start_seconds);
        let offered_for = self.position_seconds - offered_from;
        (offered_for >= 0.0).then(|| Prompt::UpNext { next, counted_down: (offered_for / UP_NEXT_COUNTDOWN_SECONDS).min(1.0) as f32 })
    }
}

#[cfg(test)]
mod tests {
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
}
