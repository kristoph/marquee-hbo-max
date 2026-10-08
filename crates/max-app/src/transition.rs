use std::time::Instant;

use crate::{
    model::ScreenPage,
    timing::{fraction_elapsed, FADE_IN, FADE_OUT},
};

pub enum Arrival {
    Page { route: String, remember: bool, page: ScreenPage },
    Play { route: String, title: String },
    Failed(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FadePhase {
    Out,
    Black,
    In,
}

pub struct Fade {
    pub phase: FadePhase,
    pub since: Instant,
    pub arrival: Option<Arrival>,
    pub arrived: Option<Instant>,
}

impl Fade {
    pub fn starting(phase: FadePhase) -> Self {
        Self { phase, since: Instant::now(), arrival: None, arrived: None }
    }

    pub fn deliver(&mut self, arrival: Arrival) {
        self.arrival = Some(arrival);
        self.arrived = Some(Instant::now());
    }

    pub fn darkness(&self) -> f32 {
        match self.phase {
            FadePhase::Out => fraction_elapsed(self.since, FADE_OUT),
            FadePhase::Black => 1.0,
            FadePhase::In => 1.0 - fraction_elapsed(self.since, FADE_IN),
        }
    }
}
