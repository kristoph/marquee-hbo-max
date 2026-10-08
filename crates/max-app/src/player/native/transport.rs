use super::NativePlayer;

const SKIP_SECONDS: f64 = 10.0;

impl NativePlayer {
    pub fn is_playing(&self) -> bool {
        self.video.is_playing()
    }

    pub fn toggle_playing(&self) {
        self.video.set_playing(!self.video.is_playing());
        self.report_progress();
    }

    pub fn position(&self) -> Option<f64> {
        self.video.position()
    }

    pub fn duration(&self) -> Option<f64> {
        self.video.duration().filter(|duration| *duration > 0.0)
    }

    pub fn seek(&self, position_seconds: f64) {
        self.video.seek(position_seconds.clamp(0.0, self.duration().unwrap_or(f64::MAX)));
    }

    pub fn skip(&self, forward: bool) {
        if let Some(position) = self.position() {
            self.seek(position + if forward { SKIP_SECONDS } else { -SKIP_SECONDS });
        }
    }
}
