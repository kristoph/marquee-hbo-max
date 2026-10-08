mod artwork;
mod command;
mod listing;

use command::Command;
use std::process::ExitCode;

use max_api::{
    client::{captured_page, Client},
    workspace,
};

type Failure = Box<dyn std::error::Error + Send + Sync>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("page-dump: {failure}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Failure> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let client = || Client::open(workspace::session());
    match Command::parse(&arguments)? {
        Command::Help => print!("{}", command::USAGE),
        Command::Captured { file, images } => show(&captured_page(file)?, false, images),
        Command::Live { route, images } => show(&client()?.page(&route)?, true, images),
        Command::PlayRoute { detail_route } => println!("{}", client()?.play_route(&detail_route)?.as_deref().unwrap_or("(nothing to play)")),
        Command::PreviewFiles { edit_id } => {
            let files = client()?.preview_files(&edit_id)?;
            println!("video {}\naudio {}", files.video, files.audio.as_deref().unwrap_or("(none)"));
        }
        Command::TitlePlayback { play_route } => {
            let playback = client()?.title_playback(&play_route)?;
            println!("certificate of {} bytes", playback.fairplay.certificate.len());
            for playlist in &playback.playlists {
                println!("{:<16} {:>6} lines", playlist.name, playlist.text.lines().count());
            }
        }
        Command::Episodes { show_id } => {
            let row = client()?.episodes(&show_id, None)?;
            println!("seasons: {}", row.filters.iter().map(|season| season.label.as_str()).collect::<Vec<_>>().join(", "));
            for episode in &row.tiles {
                println!("  {:?} {:<40} {}", episode.detail.season_and_episode, episode.title, episode.route.as_deref().unwrap_or("-"));
            }
        }
        Command::Profiles => {
            let account = client()?.account()?;
            println!("signed in: {}", account.signed_in);
            for profile in &account.profiles {
                let marks = [profile.selected.then_some("selected"), profile.needs_pin.then_some("needs a PIN")];
                println!("  {:<20} {}", profile.name, marks.into_iter().flatten().collect::<Vec<_>>().join(", "));
            }
        }
        Command::SwitchProfile { name } => {
            let client = client()?;
            let account = client.account()?;
            let profile = account.profiles.iter().find(|profile| profile.name == name).ok_or("no profile has that name")?;
            client.switch_profile(&account, &profile.id, None)?;
            println!("switched");
        }
        Command::Change { method, path } => {
            client()?.change(&method, &path, None)?;
            println!("changed");
        }
    }
    Ok(())
}

fn show(page: &max_api::cms::Page, fetched_live: bool, images: bool) {
    listing::print(page, fetched_live);
    if images {
        artwork::fetch(page);
    }
}
