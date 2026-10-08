use std::{collections::HashMap, thread, time::Instant};

use eframe::egui;

use super::App;
use crate::{
    loading::{fetch_outstanding, load_page},
    message::Message,
    model::{Outstanding, ScreenPage, Search, Selection},
    transition::{Arrival, Fade, FadePhase},
};

pub struct Navigation {
    pub route: String,
    pub history: Vec<String>,
    pub visited: HashMap<String, ScreenPage>,
    pub loading: Option<String>,
    /// Counts page changes, so that what belongs to one page (scroll positions, a preview
    /// clip) is never carried over to the next.
    pub generation: u64,
}

impl Navigation {
    pub fn starting_at(route: String) -> Self {
        Self { route, history: Vec::new(), visited: HashMap::new(), loading: None, generation: 0 }
    }

    pub fn is_first_page(&self) -> bool {
        self.generation == 0
    }
}

impl App {
    pub(crate) fn start_loading(&mut self, ctx: &egui::Context, route: String, remember: bool) {
        self.navigation.loading = Some(route.clone());
        let (sender, ctx, service) = (self.sender.clone(), ctx.clone(), self.service.clone());
        thread::spawn(move || {
            let started = Instant::now();
            let (page, outstanding) = match load_page(&service, &route) {
                Ok((page, outstanding)) => (Ok(page), outstanding),
                Err(error) => (Err(error), Outstanding::default()),
            };
            println!(
                "LOAD {route} on screen after {:.1}s; {} rows and {} images to follow",
                started.elapsed().as_secs_f32(),
                outstanding.rows.len(),
                outstanding.downloads.len()
            );
            let _ = sender.send(Message::Page { route: route.clone(), remember, page });
            ctx.request_repaint();
            fetch_outstanding(outstanding, service.client().as_deref(), &sender, &ctx);
            println!("LOAD {route} complete after {:.1}s", started.elapsed().as_secs_f32());
        });
    }

    pub(crate) fn go_to(&mut self, ctx: &egui::Context, route: String, remember: bool) {
        let fade = self.fade.insert(Fade::starting(FadePhase::Out));
        match self.navigation.visited.remove(&route) {
            Some(page) => fade.deliver(Arrival::Page { route, remember, page }),
            None => self.start_loading(ctx, route, remember),
        }
    }

    pub(crate) fn go_back(&mut self, ctx: &egui::Context) {
        if self.navigation.loading.is_none() {
            if let Some(previous) = self.navigation.history.pop() {
                self.go_to(ctx, previous, false);
            }
        }
    }

    pub(crate) fn show_page(&mut self, route: String, remember: bool, page: ScreenPage) {
        if !self.page.rows.is_empty() {
            let leaving = self.page_being_left();
            self.navigation.visited.insert(self.navigation.route.clone(), leaving);
        }
        match remember {
            true => self.navigation.history.push(std::mem::replace(&mut self.navigation.route, route)),
            false => self.navigation.route = route,
        }
        // The first page keeps the selection asked for on the command line. After that, going
        // back restores a page exactly as it was left, and going anywhere else starts at the top.
        if !self.navigation.is_first_page() {
            (self.page.selected, self.page.scroll_to) = if remember { (Selection::FIRST, None) } else { (page.selected, Some(page.scroll)) };
            self.page.stop_revealing_selection();
        }
        self.page.rows = page.rows;
        self.page.tabs = page.tabs;
        self.page.search = page.search.map(Search::new);
        self.navigation.generation += 1;
        self.hero.index = if self.page.selected.row == 0 { self.page.selected.column } else { 0 };
        self.hero.slide = None;
    }

    fn page_being_left(&mut self) -> ScreenPage {
        let search = self.page.search.take();
        let source = search.as_ref().map(Search::source);
        let tabs = self.page.tabs.take();
        match search.and_then(|search| search.rows_before_searching) {
            Some(rows) => ScreenPage { rows, tabs, selected: Selection::FIRST, scroll: 0.0, search: source },
            None => {
                ScreenPage { rows: std::mem::take(&mut self.page.rows), tabs, selected: self.page.selected, scroll: self.page.scroll, search: source }
            }
        }
    }

    pub(crate) fn forget_visited_pages(&mut self) {
        self.navigation.visited.clear();
    }
}
