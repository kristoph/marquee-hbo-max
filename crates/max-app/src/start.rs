use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use max_api::client::{Client, HOME_ROUTE};

use crate::{
    developer::{DeveloperOptions, WalkthroughMode},
    model::Selection,
    paths,
    player::PlayerChoice,
    service::Service,
};

pub const USAGE: &str = "\
max [options]

  --route <route>            start on that page instead of home, for example /series
  --select <row>[,<column>]  start with that tile selected
  --menu                     start with the browse menu open
  --about                    start with the About panel open
  --play <route>             play that title at once
  --mute                     start videos muted
  --player <web|native>      which player opens a title; the P key switches while running
  --capture                  use the saved capture of the home page instead of the network
  --session <file>           keep the signed-in session in that file
  --help-developer           list the options for checking the app without a person at it
";

pub const DEVELOPER_USAGE: &str = "\
For checking the app without a person at it:
  --background <x>,<y>       open at that screen position without taking the keyboard
  --screenshot <file.png>    save a picture of the window once artwork has loaded
  --then <route>             go on to that page three seconds after the first
  --tab <number>             open the page's tab with that number, counting from 0
  --open-menu                open the selected tile's menu
  --profiles                 open the profile picker
  --episodes                 open the episode list once an episode is playing in the native player
  --sign-in-with <file>      open the sign-in page already holding that session's cookies
  --clear-web-data           forget what the embedded web views hold before signing in
  --bench                    scroll through the page and print frame times
  --walkthrough              run the scripted walkthrough when D is pressed
  --walkthrough-now          run it unattended, ten seconds after the first page appears
  --walkthrough-dry-run      run it at once, without changing My List or playing anything
";

pub struct Start {
    pub service: Service,
    pub session_path: PathBuf,
    pub route: String,
    pub selected: Selection,
    pub browse_menu_open: bool,
    pub about_open: bool,
    pub play: Option<String>,
    pub mute_player: bool,
    pub player_choice: PlayerChoice,
    pub background_position: Option<[f32; 2]>,
    pub developer: DeveloperOptions,
}

impl Start {
    pub fn from_arguments(arguments: &[String]) -> Self {
        let has = |flag: &str| arguments.iter().any(|argument| argument == flag);
        let value_of = |flag: &str| arguments.iter().position(|argument| argument == flag).and_then(|index| arguments.get(index + 1)).cloned();
        let walkthrough = [
            ("--walkthrough-dry-run", WalkthroughMode::DryRun),
            ("--walkthrough-now", WalkthroughMode::StartUnattended),
            ("--walkthrough", WalkthroughMode::StartOnKeyPress),
        ]
        .into_iter()
        .find_map(|(flag, mode)| has(flag).then_some(mode));
        let session_path = value_of("--session").map_or_else(paths::session, PathBuf::from);
        Self {
            service: if has("--capture") { Service::Captured } else { open_service(&session_path) },
            session_path,
            route: value_of("--route").unwrap_or_else(|| HOME_ROUTE.to_string()),
            selected: value_of("--select").map(|text| selection(&text)).unwrap_or_default(),
            browse_menu_open: has("--menu"),
            about_open: has("--about"),
            play: value_of("--play"),
            mute_player: has("--mute"),
            player_choice: value_of("--player").and_then(|name| PlayerChoice::named(&name)).unwrap_or_default(),
            background_position: value_of("--background").and_then(|text| position(&text)),
            developer: DeveloperOptions {
                benchmark: has("--bench"),
                walkthrough,
                screenshot_path: value_of("--screenshot"),
                route_to_go_on_to: value_of("--then"),
                open_tile_menu: has("--open-menu"),
                tab_to_open: value_of("--tab").and_then(|number| number.parse().ok()),
                open_profile_picker: has("--profiles"),
                open_episodes: has("--episodes"),
                clear_web_data: has("--clear-web-data"),
                session_to_sign_in_with: value_of("--sign-in-with").map(PathBuf::from),
            },
        }
    }
}

fn open_service(session_path: &Path) -> Service {
    match Client::open(session_path) {
        Ok(client) => Service::Live(Arc::new(client)),
        Err(_) => Service::SignedOut,
    }
}

fn selection(text: &str) -> Selection {
    let mut numbers = text.split(',').map(|number| number.parse().unwrap_or(0));
    Selection::new(numbers.next().unwrap_or(0), numbers.next().unwrap_or(0))
}

fn position(text: &str) -> Option<[f32; 2]> {
    let (x, y) = text.split_once(',')?;
    Some([x.parse().ok()?, y.parse().ok()?])
}
