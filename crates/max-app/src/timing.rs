use std::time::Duration;

pub const FADE_OUT: Duration = Duration::from_millis(180);
pub const FADE_IN: Duration = Duration::from_millis(260);
pub const SPINNER_DELAY: Duration = Duration::from_millis(150);
pub const WAIT_FOR_FIRST_SCREEN_ARTWORK: Duration = Duration::from_millis(1500);
pub const HERO_SLIDE: Duration = Duration::from_millis(450);
pub const HERO_DWELL_WITHOUT_PREVIEW_SECONDS: f32 = 8.0;
pub const PREVIEW_DELAY: Duration = Duration::from_millis(2500);
pub const PREVIEW_FADE: Duration = Duration::from_millis(600);
pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(350);
pub const ARTWORK_FADE_SECONDS: f32 = 0.18;
pub const SELECTION_GROW_SECONDS: f32 = 0.12;
pub const TOAST: Duration = Duration::from_millis(2600);
pub const COOKIE_STORE_SETTLING: Duration = Duration::from_millis(350);

pub fn fraction_elapsed(since: std::time::Instant, of: Duration) -> f32 {
    (since.elapsed().as_secs_f32() / of.as_secs_f32()).clamp(0.0, 1.0)
}
