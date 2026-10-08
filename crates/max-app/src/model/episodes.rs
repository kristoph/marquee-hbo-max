use max_api::cms::Filter;

use super::rail::ScreenTile;

pub struct LoadedEpisodes {
    pub seasons: Vec<Filter>,
    pub episodes: Vec<ScreenTile>,
}

/// The list of a series' episodes shown over the title that is playing.
pub struct EpisodesPanel {
    pub show_id: String,
    pub seasons: Vec<Filter>,
    pub season_number: Option<u32>,
    pub episodes: Option<Vec<ScreenTile>>,
}

impl EpisodesPanel {
    pub fn loading(show_id: String, season_number: Option<u32>) -> Self {
        Self { show_id, seasons: Vec::new(), season_number, episodes: None }
    }

    pub fn season_parameters(&self) -> Option<String> {
        self.season_number.map(|number| format!("pf[seasonNumber]={number}"))
    }

    pub fn is_selected(&self, season: &Filter) -> bool {
        self.season_number.is_some_and(|number| season.label == number.to_string())
    }

    pub fn show(&mut self, loaded: LoadedEpisodes) {
        if self.season_number.is_none() {
            self.season_number = loaded.episodes.first().and_then(|episode| episode.detail.season_and_episode).map(|(season, _)| season);
        }
        self.seasons = loaded.seasons;
        self.episodes = Some(loaded.episodes);
    }
}

#[cfg(test)]
mod tests {
    use max_api::cms::TileDetail;

    use super::*;

    fn episode(season: u32, number: u32) -> ScreenTile {
        let detail = TileDetail { season_and_episode: Some((season, number)), ..TileDetail::default() };
        ScreenTile { title: String::new(), route: None, artwork: None, logo: None, badge_icon: None, banner_icon: None, detail }
    }

    fn season(label: &str) -> Filter {
        Filter { label: label.to_string(), parameter: format!("pf[seasonNumber]={label}") }
    }

    #[test]
    fn asks_for_the_season_being_watched() {
        let panel = EpisodesPanel::loading("show".to_string(), Some(3));
        assert_eq!(panel.season_parameters().as_deref(), Some("pf[seasonNumber]=3"));
        assert!(panel.is_selected(&season("3")) && !panel.is_selected(&season("1")));
    }

    #[test]
    fn takes_the_season_from_what_the_service_chose_when_none_was_asked_for() {
        let mut panel = EpisodesPanel::loading("show".to_string(), None);
        assert_eq!(panel.season_parameters(), None);
        panel.show(LoadedEpisodes { seasons: vec![season("1"), season("2")], episodes: vec![episode(2, 1), episode(2, 2)] });
        assert_eq!(panel.season_number, Some(2));
        assert_eq!(panel.episodes.as_ref().map(Vec::len), Some(2));
    }
}
