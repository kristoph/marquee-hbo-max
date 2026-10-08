use max_api::cms::NextVideo;

use super::NativePlayer;
use crate::player::prompts::{Moment, Prompt};

impl NativePlayer {
    pub fn prompt(&self) -> Option<Prompt<'_>> {
        let moment = Moment {
            position_seconds: self.position()?,
            duration_seconds: self.duration()?,
            sections: &self.sections,
            next: self.next.as_ref(),
            watching_credits: self.watching_credits.get(),
        };
        moment.prompt()
    }

    pub fn watch_credits(&self) {
        self.watching_credits.set(true);
    }

    /// The next episode starts by itself when its wait has run out, or when the title ends
    /// before that; someone who chose the credits is left with them.
    pub fn next_when_due(&self) -> Option<&NextVideo> {
        let waited_out = matches!(self.prompt(), Some(Prompt::UpNext { counted_down, .. }) if counted_down >= 1.0);
        self.next.as_ref().filter(|_| waited_out || (self.finished() && !self.watching_credits.get()))
    }
}
