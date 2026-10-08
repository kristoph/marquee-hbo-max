use super::NativePlayer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound {
    pub muted: bool,
    pub volume: f32,
}

impl NativePlayer {
    pub fn is_muted(&self) -> bool {
        self.muted.get()
    }

    pub fn volume(&self) -> f32 {
        self.volume.get()
    }

    /// Moving the volume off silence unmutes, as a person would expect of a slider.
    pub fn set_volume(&self, volume: f32) {
        self.volume.set(volume);
        self.video.set_volume(volume);
        self.muted.set(volume <= 0.0);
        self.video.set_muted(self.muted.get());
    }

    pub fn sound(&self) -> Sound {
        Sound { muted: self.muted.get(), volume: self.volume.get() }
    }

    pub fn toggle_muted(&self) {
        self.muted.set(!self.muted.get());
        self.video.set_muted(self.muted.get());
    }
}
