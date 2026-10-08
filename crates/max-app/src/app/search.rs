use std::{thread, time::Duration};

use eframe::egui;
use max_api::Error;

use super::App;
use crate::{
    loading::fetch_outstanding,
    message::{Message, SearchResults},
    model::{grid_lines, Outstanding, ScreenBuilder, SearchQuery, Selection},
    timing::SEARCH_DEBOUNCE,
};

const DEBOUNCE_POLL: Duration = Duration::from_millis(50);

impl App {
    pub(crate) fn run_search(&mut self, ctx: &egui::Context) {
        let Some(search) = &mut self.page.search else { return };
        match search.edited {
            Some(edited) if edited.elapsed() < SEARCH_DEBOUNCE => return ctx.request_repaint_after(DEBOUNCE_POLL),
            _ => search.edited = None,
        }
        let query = search.wanted();
        if query == search.shown {
            return;
        }
        search.shown = query.clone();
        if query.is_empty() {
            search.topics = search.starting_topics.clone();
            if let Some(rows) = search.rows_before_searching.take() {
                self.page.rows = rows;
                self.page.selected = Selection::FIRST;
                self.navigation.generation += 1;
            }
            return;
        }
        let Some(client) = self.service.client() else { return };
        let (sender, ctx, results_id) = (self.sender.clone(), ctx.clone(), search.results_id.clone());
        thread::spawn(move || {
            let built = client.search(&results_id, &query.text, query.topic_parameters.as_deref()).map(|row| {
                let mut builder = ScreenBuilder::new(true);
                (SearchResults { row: builder.row(&row), topics: row.filters }, builder.outstanding)
            });
            let (results, outstanding) = match built {
                Ok((results, outstanding)) => (Ok(results), outstanding),
                Err(error) => (Err(error), Outstanding::default()),
            };
            let _ = sender.send(Message::SearchResults { query, results });
            ctx.request_repaint();
            fetch_outstanding(outstanding, Some(&client), &sender, &ctx);
        });
    }

    pub(crate) fn show_search_results(&mut self, query: &SearchQuery, results: Result<SearchResults, Error>) {
        let Some(search) = &mut self.page.search else { return };
        if search.shown != *query {
            return;
        }
        let results = match results {
            Ok(results) if !results.row.tiles.is_empty() => results,
            Ok(_) => return self.say(format!("Nothing found for “{}”", query.text)),
            Err(error) => return self.failed("Search failed", &error),
        };
        // Typing brings its own topics; picking a topic keeps the list it was picked from.
        if query.topic_parameters.is_none() && !results.topics.is_empty() {
            search.topics = results.topics;
        }
        let lines = grid_lines(results.row.tiles, results.row.layout, self.page.width);
        let previous = std::mem::replace(&mut self.page.rows, lines);
        search.rows_before_searching.get_or_insert(previous);
        self.page.selected = Selection::FIRST;
        self.navigation.generation += 1;
    }

    pub(crate) fn lay_grid_out_again(&mut self, width: f32) {
        let Some(layout) = self.page.rows.first().filter(|row| row.grid_line).map(|row| row.layout) else { return };
        if crate::model::tiles_across(layout, width) == crate::model::tiles_across(layout, self.page.width) {
            return;
        }
        let tiles = std::mem::take(&mut self.page.rows).into_iter().flat_map(|row| row.tiles).collect();
        self.page.rows = grid_lines(tiles, layout, width);
        self.page.selected = Selection::FIRST;
    }
}
