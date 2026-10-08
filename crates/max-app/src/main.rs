mod app;
mod artwork;
mod developer;
mod hero;
mod intent;
mod loading;
mod message;
mod metrics;
mod model;
mod paint;
mod paths;
mod player;
mod routes;
mod service;
mod signin;
mod start;
mod theme;
mod timing;
mod transition;
mod view;
mod web;

use eframe::egui::ViewportBuilder;

use crate::{app::App, start::Start};

/// What the app itself can fail at is only ever reported, never told apart.
type Failure = Box<dyn std::error::Error + Send + Sync>;

const WINDOW_TITLE: &str = "HBO Max";
const ICON: &[u8] = include_bytes!("../assets/icon.png");
const WINDOW_SIZE: [f32; 2] = [1600.0, 1000.0];
/// Without a position the window can open on a secondary display.
const WINDOW_POSITION: [f32; 2] = [300.0, 120.0];

fn main() -> eframe::Result {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|argument| argument == "--help") {
        print!("{}", start::USAGE);
        return Ok(());
    }
    let start = Start::from_arguments(&arguments);
    let viewport = ViewportBuilder::default()
        .with_inner_size(WINDOW_SIZE)
        .with_position(start.background_position.unwrap_or(WINDOW_POSITION))
        .with_title(WINDOW_TITLE)
        .with_icon(eframe::icon_data::from_png_bytes(ICON).expect("the icon built into the app is a PNG"))
        .with_transparent(true);
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("max", options, Box::new(move |creation_context| Ok(Box::new(App::new(creation_context, start)))))
}
