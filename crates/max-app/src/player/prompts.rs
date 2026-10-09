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
mod tests;
