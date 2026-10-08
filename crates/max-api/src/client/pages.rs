use std::{fs, path::Path, thread::sleep, time::Duration};

use super::Client;
use crate::{
    cms::{Document, Page, Row},
    Error,
};

pub const HOME_ROUTE: &str = "/home";
pub(super) const COLLECTION_QUERY: &str = "include=default&decorators=viewingHistory,isFavorite,contentAction,badges";
const PAUSE_BETWEEN_ROWS: Duration = Duration::from_millis(250);

impl Client {
    pub fn page_outline(&self, route: &str) -> Result<Page, Error> {
        let document = Document::parse(&self.get(&self.session.page_url(route))?)?;
        document.page().ok_or(Error::Lacks("a page"))
    }

    pub fn page(&self, route: &str) -> Result<Page, Error> {
        let mut page = self.page_outline(route)?;
        self.fill_deferred_rows(&mut page.rows);
        for group in &mut page.rows {
            if let Some(first_tab) = group.tabs.first_mut() {
                self.fill_deferred_rows(&mut first_tab.rows);
            }
        }
        Ok(page)
    }

    pub fn row(&self, id: &str, mandatory_parameters: Option<&str>) -> Result<Row, Error> {
        let mut url = format!("{}/cms/collections/{id}?{COLLECTION_QUERY}", self.session.content_origin());
        if let Some(parameters) = mandatory_parameters {
            url.push('&');
            url.push_str(parameters);
        }
        Document::parse(&self.get(&url)?)?.collection().ok_or(Error::Lacks("a collection"))
    }

    pub fn play_route(&self, detail_route: &str) -> Result<Option<String>, Error> {
        Ok(play_route_of(&Document::parse(&self.get(&self.session.page_url(detail_route))?)?))
    }

    fn fill_deferred_rows(&self, rows: &mut [Row]) {
        for row in rows.iter_mut().filter(|row| row.deferred && row.tiles.is_empty()) {
            sleep(PAUSE_BETWEEN_ROWS);
            match self.row(&row.id, row.mandatory_parameters.as_deref()) {
                Ok(fetched) => row.tiles = fetched.tiles,
                Err(error) => log::warn!("row {:?} failed: {error}", row.title),
            }
        }
    }
}

pub fn captured_page(path: impl AsRef<Path>) -> Result<Page, Error> {
    Document::parse(&fs::read_to_string(path)?)?.page().ok_or(Error::Lacks("a page"))
}

/// Later playing actions on a detail page are trailers and extras; the first is the title itself.
fn play_route_of(detail: &Document) -> Option<String> {
    let page = detail.page()?;
    let lead = page.rows.first()?.tiles.first()?;
    lead.detail.actions.iter().find(|action| action.plays()).and_then(|action| action.route.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail_page(listed_actions: &[&str]) -> Document {
        let listed: Vec<String> = listed_actions.iter().map(|id| format!(r#"{{"type": "userAction", "id": "{id}"}}"#)).collect();
        Document::parse(&format!(
            r#"{{
              "data": {{"type": "route", "id": "r", "relationships": {{"target": {{"data": {{"type": "page", "id": "p"}}}}}}}},
              "included": [
                {{"type": "page", "id": "p", "relationships": {{"items": {{"data": [{{"type": "pageItem", "id": "pi"}}]}}}}}},
                {{"type": "pageItem", "id": "pi", "relationships": {{"collection": {{"data": {{"type": "collection", "id": "c"}}}}}}}},
                {{"type": "collection", "id": "c", "attributes": {{"component": {{"id": "hero"}}}},
                  "relationships": {{"items": {{"data": [{{"type": "collectionItem", "id": "ci"}}]}}}}}},
                {{"type": "collectionItem", "id": "ci",
                  "relationships": {{"show": {{"data": {{"type": "show", "id": "s"}}}}, "userActions": {{"data": [{}]}}}}}},
                {{"type": "show", "id": "s", "attributes": {{"name": "A Show"}}}},
                {{"type": "userAction", "id": "info", "attributes": {{"context": "generic", "elements": {{"label": {{"label": "More Info"}}}}}},
                  "relationships": {{"route": {{"data": {{"type": "route", "id": "r-info"}}}}}}}},
                {{"type": "userAction", "id": "resume", "attributes": {{"context": "resume", "elements": {{"label": {{"label": "Resume S1 E3"}}}}}},
                  "relationships": {{"route": {{"data": {{"type": "route", "id": "r-ep"}}}}}}}},
                {{"type": "userAction", "id": "trailer", "attributes": {{"context": "play", "elements": {{"label": {{"label": "Trailer"}}}}}},
                  "relationships": {{"route": {{"data": {{"type": "route", "id": "r-trailer"}}}}}}}},
                {{"type": "route", "id": "r-info", "attributes": {{"url": "/show/s"}}}},
                {{"type": "route", "id": "r-ep", "attributes": {{"url": "/video/watch/ep3"}}}},
                {{"type": "route", "id": "r-trailer", "attributes": {{"url": "/video/watch/trailer"}}}}
              ]
            }}"#,
            listed.join(",")
        ))
        .unwrap()
    }

    #[test]
    fn the_episode_to_resume_wins_over_the_trailer_listed_after_it() {
        assert_eq!(play_route_of(&detail_page(&["info", "resume", "trailer"])).as_deref(), Some("/video/watch/ep3"));
    }

    #[test]
    fn a_page_with_nothing_to_play_has_no_play_route() {
        assert_eq!(play_route_of(&detail_page(&["info"])), None);
    }
}
