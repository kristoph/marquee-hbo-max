use std::time::{Duration, Instant};

use objc2::rc::Retained;
use objc2_av_foundation::{AVPlayerItem, AVPlayerItemStatus};
use objc2_core_media::CMClock;

use super::{clip::Clip, time::seconds};

const TOLERATED_DRIFT_SECONDS: f64 = 0.2;
const DRIFT_CHECK_INTERVAL: Duration = Duration::from_secs(2);

impl Clip {
    pub fn set_muted(&self, muted: bool) {
        self.muted.set(muted);
        self.apply_sound_state();
    }

    pub fn restart_sound_in_step(&self) {
        if self.sound_wanted() {
            self.start_sound();
        }
    }

    pub(super) fn apply_sound_state(&self) {
        match self.sound_wanted() {
            true => self.start_sound(),
            false => self.stop_sound(),
        }
    }

    pub(super) fn keep_sound_in_step(&self) {
        if self.sound_wanted() && (!self.sound_started.get() || self.sound_has_drifted()) {
            self.start_sound();
        }
    }

    fn sound_wanted(&self) -> bool {
        !self.muted.get() && self.playing.get()
    }

    fn stop_sound(&self) {
        self.sound_started.set(false);
        if let Some(sound) = &self.sound {
            // SAFETY: on the main thread.
            unsafe { sound.pause() }
        }
    }

    /// Tells the sound's player which moment of the clip belongs to the present instant on the
    /// system clock. Seeking the sound to the picture's time instead lands near it, not on it,
    /// and correcting that repeatedly is audible as stutter.
    ///
    /// The player raises an exception if asked before both files can play, so until then this
    /// does nothing and `keep_sound_in_step` asks again.
    fn start_sound(&self) {
        let Some(sound) = &self.sound else { return };
        self.sound_checked.set(Instant::now());
        let ready = |item: Option<Retained<AVPlayerItem>>| {
            // SAFETY: on the main thread.
            item.is_some_and(|item| unsafe { item.status() } == AVPlayerItemStatus::ReadyToPlay)
        };
        // SAFETY: on the main thread.
        unsafe {
            if !ready(Some(self.item.clone())) || !ready(sound.currentItem()) {
                return;
            }
            let (clip_time, clock_time) = (self.item.currentTime(), CMClock::host_time_clock().time());
            let started = objc2::exception::catch(std::panic::AssertUnwindSafe(|| sound.setRate_time_atHostTime(1.0, clip_time, clock_time)));
            match started {
                Ok(()) => self.sound_started.set(true),
                Err(exception) => eprintln!("preview sound could not start: {exception:?}"),
            }
        }
    }

    fn sound_has_drifted(&self) -> bool {
        let Some(sound) = &self.sound else { return false };
        if self.sound_checked.get().elapsed() < DRIFT_CHECK_INTERVAL {
            return false;
        }
        self.sound_checked.set(Instant::now());
        // SAFETY: on the main thread.
        let (seen, heard) = unsafe { (seconds(self.item.currentTime()), seconds(sound.currentTime())) };
        matches!((seen, heard), (Some(seen), Some(heard)) if (seen - heard).abs() > TOLERATED_DRIFT_SECONDS)
    }
}
