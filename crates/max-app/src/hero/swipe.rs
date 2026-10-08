//! A trackpad keeps sending scroll events after the fingers lift (momentum), often for more
//! than a second. Waiting for those to stop before accepting the next swipe makes quick
//! back-and-forth swiping feel stuck, so a new swipe is also recognised inside the tail of the
//! last one: by a change of direction, which momentum never has, or by the scrolling picking
//! up speed again after it had been dying away.

use std::time::{Duration, Instant};

const DISTANCE_OF_A_SWIPE: f32 = 100.0;
const PAUSE_THAT_ENDS_A_SWIPE: Duration = Duration::from_millis(250);
const SMALLEST_EVENT_THAT_STARTS_A_SWIPE: f32 = 5.0;
const COASTING_BELOW_FRACTION_OF_PEAK: f32 = 0.5;
const RESURGENCE_OVER_LOWEST: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
struct CountedSwipe {
    forward: bool,
    largest_event: f32,
    smallest_event_while_coasting: Option<f32>,
}

#[derive(Debug, Default)]
pub struct Swipes {
    travelled: f32,
    counted: Option<CountedSwipe>,
    last_event: Option<Instant>,
}

impl Swipes {
    pub fn is_active(&self, now: Instant) -> bool {
        self.last_event.is_some_and(|at| now.duration_since(at) < PAUSE_THAT_ENDS_A_SWIPE)
    }

    /// Takes one scroll event's sideways movement, positive moving content to the right, and
    /// returns whether it completed a swipe forward (to the left) or back.
    pub fn feed(&mut self, sideways: f32, now: Instant) -> Option<bool> {
        if sideways == 0.0 {
            return None;
        }
        if !self.is_active(now) {
            *self = Self::default();
        }
        self.last_event = Some(now);

        let (size, forward) = (sideways.abs(), sideways < 0.0);
        if let Some(counted) = &mut self.counted {
            let reversed = forward != counted.forward && size >= SMALLEST_EVENT_THAT_STARTS_A_SWIPE;
            counted.largest_event = counted.largest_event.max(size);
            if size <= counted.largest_event * COASTING_BELOW_FRACTION_OF_PEAK {
                counted.smallest_event_while_coasting = Some(counted.smallest_event_while_coasting.map_or(size, |smallest| smallest.min(size)));
            }
            let resurgent = counted
                .smallest_event_while_coasting
                .is_some_and(|smallest| size >= SMALLEST_EVENT_THAT_STARTS_A_SWIPE && size >= smallest * RESURGENCE_OVER_LOWEST);
            if !(reversed || resurgent) {
                return None;
            }
            self.counted = None;
            self.travelled = 0.0;
        }

        self.travelled += sideways;
        if self.travelled.abs() < DISTANCE_OF_A_SWIPE {
            return None;
        }
        let forward = self.travelled < 0.0;
        self.counted = Some(CountedSwipe { forward, largest_event: size, smallest_event_while_coasting: None });
        self.travelled = 0.0;
        Some(forward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(swipes: &mut Swipes, start: Instant, events: &[f32]) -> (Vec<bool>, Instant) {
        let mut now = start;
        let mut steps = Vec::new();
        for sideways in events {
            steps.extend(swipes.feed(*sideways, now));
            now += Duration::from_millis(10);
        }
        (steps, now)
    }
    const LEFT: [f32; 22] = [
        -8.0, -20.0, -35.0, -50.0, -60.0, -55.0, -48.0, -40.0, -33.0, -27.0, -22.0, -18.0, -14.0, -11.0, -9.0, -7.0, -5.0, -4.0, -3.0, -2.0, -1.0,
        -1.0,
    ];

    fn right() -> Vec<f32> {
        LEFT.iter().map(|sideways| -sideways).collect()
    }

    #[test]
    fn one_swipe_is_one_step_whatever_its_momentum() {
        let mut swipes = Swipes::default();
        let (steps, _) = run(&mut swipes, Instant::now(), &LEFT);
        assert_eq!(steps, [true]);
    }

    #[test]
    fn a_small_nudge_is_not_a_swipe() {
        let mut swipes = Swipes::default();
        let (steps, _) = run(&mut swipes, Instant::now(), &[-10.0, -15.0, -12.0, -6.0, -3.0, -1.0]);
        assert!(steps.is_empty());
    }

    #[test]
    fn reversing_during_the_momentum_counts_at_once() {
        let mut swipes = Swipes::default();
        let (first, now) = run(&mut swipes, Instant::now(), &LEFT[..12]);
        let (second, now) = run(&mut swipes, now, &right()[..12]);
        let (third, _) = run(&mut swipes, now, &LEFT[..12]);
        assert_eq!((first, second, third), (vec![true], vec![false], vec![true]));
    }

    #[test]
    fn a_second_swipe_the_same_way_during_the_momentum_counts() {
        let mut swipes = Swipes::default();
        let (first, now) = run(&mut swipes, Instant::now(), &LEFT[..17]);
        let (second, _) = run(&mut swipes, now, &LEFT);
        assert_eq!((first, second), (vec![true], vec![true]));
    }

    #[test]
    fn a_swipe_still_speeding_up_when_counted_is_not_counted_twice() {
        let mut swipes = Swipes::default();
        let events = [-30.0, -30.0, -30.0, -30.0, -45.0, -70.0, -110.0, -150.0, -120.0, -80.0, -50.0, -30.0, -15.0, -6.0, -2.0];
        let (steps, _) = run(&mut swipes, Instant::now(), &events);
        assert_eq!(steps, [true]);
    }

    #[test]
    fn a_pause_starts_afresh() {
        let mut swipes = Swipes::default();
        let (first, now) = run(&mut swipes, Instant::now(), &LEFT);
        assert!(!swipes.is_active(now + PAUSE_THAT_ENDS_A_SWIPE));
        let (second, _) = run(&mut swipes, now + PAUSE_THAT_ENDS_A_SWIPE, &LEFT);
        assert_eq!((first, second), (vec![true], vec![true]));
    }
}
