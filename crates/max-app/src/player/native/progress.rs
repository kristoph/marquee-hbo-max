use std::{
    thread,
    time::{Duration, Instant},
};

use super::NativePlayer;

const REPORT_PROGRESS_EVERY: Duration = Duration::from_secs(20);
const LOG_PROGRESS_EVERY: Duration = Duration::from_secs(5);

impl NativePlayer {
    pub fn report_progress_when_due(&mut self) {
        if self.is_playing() && self.progress_reported.elapsed() >= REPORT_PROGRESS_EVERY {
            self.progress_reported = Instant::now();
            self.report_progress();
        }
    }

    pub fn report_progress(&self) {
        let Some(position) = self.position() else { return };
        let (client, watching) = (self.client.clone(), self.watching.clone());
        thread::spawn(move || {
            if let Err(error) = client.report_progress(&watching, position) {
                eprintln!("progress could not be reported: {error}");
            }
        });
    }

    pub fn log_progress_when_due(&mut self) {
        if self.progress_logged.elapsed() >= LOG_PROGRESS_EVERY {
            self.progress_logged = Instant::now();
            println!("NATIVE PLAYER {}", self.progress_report());
        }
    }

    fn progress_report(&self) -> String {
        let seconds = |value: Option<f64>| value.map_or_else(|| "?".to_string(), |value| format!("{value:.1}s"));
        format!("at {} of {}, picture ready: {}", seconds(self.position()), seconds(self.duration()), self.video.has_picture())
    }
}
