use std::path::PathBuf;

use max_api::{client::HOME_ROUTE, workspace};

use crate::Failure;

const HOME_CAPTURE: &str = "routes-home";

pub const USAGE: &str = "\
page-dump [file]                    list a saved page response (the home capture by default)
page-dump --live [route]            fetch and list a page with the saved session
page-dump --play-route <route>      print the route that plays a title, given its page
page-dump --preview-files <edit id> print the files of a hero preview clip
page-dump --title-playback <route>  list the playlists written for a title's playback route
page-dump --episodes <show id>      list a series' seasons and the episodes of the one the service picks
page-dump --profiles                list the account's profiles
page-dump --switch-profile <name>   make that profile the one in use
page-dump --change <METHOD> <path>  make an account change, as a tile's menu would

Add --images to a listing to download the artwork it would show.
";

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Captured { file: PathBuf, images: bool },
    Live { route: String, images: bool },
    PlayRoute { detail_route: String },
    PreviewFiles { edit_id: String },
    TitlePlayback { play_route: String },
    Episodes { show_id: String },
    Profiles,
    SwitchProfile { name: String },
    Change { method: String, path: String },
}

impl Command {
    pub fn parse(arguments: &[String]) -> Result<Self, Failure> {
        let images = arguments.iter().any(|argument| argument == "--images");
        let mut words = arguments.iter().filter(|argument| *argument != "--images").map(String::as_str);
        let mut value = |what: &str| words.next().map(str::to_string).ok_or_else(|| Failure::from(format!("this needs {what}; see --help")));
        let command = match value("a command") {
            Err(_) => Self::Captured { file: workspace::capture(HOME_CAPTURE), images },
            Ok(flag) => match flag.as_str() {
                "--help" => Self::Help,
                "--live" => Self::Live { route: value("a route").unwrap_or_else(|_| HOME_ROUTE.to_string()), images },
                "--play-route" => Self::PlayRoute { detail_route: value("the route of a title's page")? },
                "--preview-files" => Self::PreviewFiles { edit_id: value("the id of a preview clip's edit")? },
                "--title-playback" => Self::TitlePlayback { play_route: value("a playback route")? },
                "--episodes" => Self::Episodes { show_id: value("a series' id")? },
                "--profiles" => Self::Profiles,
                "--switch-profile" => Self::SwitchProfile { name: value("a profile's name")? },
                "--change" => Self::Change { method: value("a method")?, path: value("a path")? },
                file if !file.starts_with("--") => Self::Captured { file: PathBuf::from(file), images },
                unknown => return Err(format!("{unknown} is not an option; see --help").into()),
            },
        };
        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<Command, Failure> {
        Command::parse(&line.split_whitespace().map(str::to_string).collect::<Vec<_>>())
    }

    #[test]
    fn with_nothing_asked_it_lists_the_saved_home_page() {
        assert_eq!(parse("").unwrap(), Command::Captured { file: workspace::capture(HOME_CAPTURE), images: false });
        assert_eq!(parse("saved.json --images").unwrap(), Command::Captured { file: PathBuf::from("saved.json"), images: true });
    }

    #[test]
    fn reads_each_command_with_its_values() {
        assert_eq!(parse("--live").unwrap(), Command::Live { route: HOME_ROUTE.to_string(), images: false });
        assert_eq!(parse("--images --live /series").unwrap(), Command::Live { route: "/series".to_string(), images: true });
        assert_eq!(
            parse("--change DELETE /my-list/show/x").unwrap(),
            Command::Change { method: "DELETE".to_string(), path: "/my-list/show/x".to_string() }
        );
    }

    #[test]
    fn says_what_is_missing_or_unknown() {
        assert!(parse("--episodes").unwrap_err().to_string().contains("a series' id"));
        assert!(parse("--nonsense").unwrap_err().to_string().contains("not an option"));
    }
}
