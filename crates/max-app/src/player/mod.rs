mod native;
mod prompts;
mod web;

pub use native::{NativePlayer, Sound};
pub use prompts::Prompt;
pub use web::{Player, PlayerOptions};

/// Which player opens a title. The native one is being built; the web one is the service's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerChoice {
    #[default]
    Web,
    Native,
}

impl PlayerChoice {
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "web" => Some(Self::Web),
            "native" => Some(Self::Native),
            _ => None,
        }
    }

    pub fn other(self) -> Self {
        match self {
            Self::Web => Self::Native,
            Self::Native => Self::Web,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Web => "Player: the service's web player",
            Self::Native => "Player: native (in development)",
        }
    }
}

pub struct PendingPlayback {
    pub route: String,
    pub title: String,
}

/// At most one of the two players is open at a time.
pub struct Playback {
    pub web: Option<Player>,
    pub native: Option<NativePlayer>,
    pub choice: PlayerChoice,
    pub sound: Sound,
    pub to_open: Option<PendingPlayback>,
}

impl Playback {
    pub fn new(choice: PlayerChoice, muted: bool) -> Self {
        Self { web: None, native: None, choice, sound: Sound { muted, volume: 1.0 }, to_open: None }
    }

    pub fn is_open(&self) -> bool {
        self.web.is_some() || self.native.is_some()
    }

    pub fn close(&mut self) {
        self.web = None;
        self.native = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_choice_of_player_is_named_and_switches_back_and_forth() {
        assert_eq!(PlayerChoice::named("native"), Some(PlayerChoice::Native));
        assert_eq!(PlayerChoice::named("vlc"), None);
        assert_eq!(PlayerChoice::default().other().other(), PlayerChoice::Web);
    }
}
