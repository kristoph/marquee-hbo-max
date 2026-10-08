use super::Client;
use crate::{cms::Row, Error};

const EPISODES_COLLECTION: &str = "generic-show-page-rail-episodes-tabbed-content";

impl Client {
    /// The episodes of one season as a row, whose filters are the series' seasons. Without a
    /// season the service picks the one to show.
    pub fn episodes(&self, show_id: &str, season_parameters: Option<&str>) -> Result<Row, Error> {
        let show = format!("pf[show.id]={show_id}");
        let parameters: Vec<&str> = std::iter::once(show.as_str()).chain(season_parameters).collect();
        self.row(EPISODES_COLLECTION, Some(&parameters.join("&")))
    }
}
