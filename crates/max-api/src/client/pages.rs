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
mod tests;
