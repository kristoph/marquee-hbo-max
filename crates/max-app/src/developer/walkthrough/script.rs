use max_api::client::HOME_ROUTE;

use crate::routes;

#[derive(Clone, Copy)]
pub enum Step {
    NextHeroTitle,
    DownRows(usize),
    AcrossToColumn(usize),
    TopOfPage,
    GoTo(&'static str),
    Type(&'static str),
    SelectTitle(&'static str),
    OpenTileMenu,
    MoveToMyListLine,
    AddToMyList,
    CloseTileMenu,
    Play,
    Back,
}

pub struct ScheduledStep {
    pub at_seconds: f32,
    pub step: Step,
}

const fn at(at_seconds: f32, step: Step) -> ScheduledStep {
    ScheduledStep { at_seconds, step }
}

/// Timed to the recorded voiceover; the quotations are the lines each group of steps falls under.
pub const SCRIPT: &[ScheduledStep] = &[
    // "This should look familiar."
    at(13.0, Step::NextHeroTitle),
    // "It's a fully native HBO Max client…"
    at(16.0, Step::DownRows(2)),
    at(18.0, Step::DownRows(2)),
    at(20.0, Step::DownRows(2)),
    at(22.0, Step::TopOfPage),
    // "It's fast, responsive…" and "The content views"
    at(23.2, Step::GoTo("/movies")),
    at(25.6, Step::DownRows(2)),
    at(26.8, Step::DownRows(2)),
    at(28.0, Step::DownRows(2)),
    at(29.4, Step::DownRows(2)),
    // "search"
    at(31.0, Step::GoTo(routes::SEARCH)),
    at(32.2, Step::Type("superman")),
    at(34.2, Step::SelectTitle("Superman")),
    // "preferences, and bookmarks are all here"
    at(35.0, Step::OpenTileMenu),
    at(35.9, Step::MoveToMyListLine),
    at(36.8, Step::AddToMyList),
    at(38.4, Step::GoTo(routes::MY_STUFF)),
    // "This is a fully working application…"
    at(40.6, Step::SelectTitle("Superman")),
    at(41.6, Step::Play),
    // "I've been assembling a development pipeline…"
    at(57.0, Step::Back),
    at(59.5, Step::GoTo(HOME_ROUTE)),
    at(63.5, Step::NextHeroTitle),
    at(66.5, Step::DownRows(1)),
    at(67.3, Step::AcrossToColumn(2)),
    at(68.5, Step::OpenTileMenu),
    at(71.0, Step::CloseTileMenu),
    at(72.5, Step::TopOfPage),
    // "This is the fourth HBO Max client…"
    at(75.0, Step::GoTo("/series")),
    at(78.0, Step::DownRows(2)),
    at(80.0, Step::DownRows(2)),
    at(82.0, Step::DownRows(2)),
    at(85.0, Step::GoTo(HOME_ROUTE)),
    // "AI isn't just helping us write better code faster…"
    at(89.5, Step::NextHeroTitle),
    at(101.0, Step::TopOfPage),
];
