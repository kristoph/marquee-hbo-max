use std::thread;

use eframe::egui;

use super::App;
use crate::{
    loading::fetch_outstanding,
    model::{ScreenBuilder, ScreenRow, Selection},
    transition::Arrival,
};

impl App {
    pub(crate) fn open_tab(&mut self, ctx: &egui::Context, index: usize) {
        let Some(tabs) = &mut self.page.tabs else { return };
        if index == tabs.selected || index >= tabs.titles.len() {
            return;
        }
        if let Some(rows) = tabs.opened[index].take() {
            return self.show_tab(index, rows);
        }
        let source = tabs.unopened[index].take().unwrap_or_default();
        let mut builder = ScreenBuilder::new(self.service.is_live());
        let rows = builder.rows(&source);
        self.show_tab(index, rows);
        let (sender, ctx, client, outstanding) = (self.sender.clone(), ctx.clone(), self.service.client(), builder.outstanding);
        thread::spawn(move || fetch_outstanding(outstanding, client.as_deref(), &sender, &ctx));
    }

    fn show_tab(&mut self, index: usize, rows: Vec<ScreenRow>) {
        let Some(tabs) = &mut self.page.tabs else { return };
        let rows_shown = rows.len();
        let previous: Vec<ScreenRow> = self.page.rows.splice(tabs.first_row..tabs.first_row + tabs.rows_shown, rows).collect();
        tabs.opened[tabs.selected] = Some(previous);
        tabs.opened[index] = Some(Vec::new());
        tabs.selected = index;
        tabs.rows_shown = rows_shown;
        self.page.selected = Selection::new(tabs.first_row.min(self.page.rows.len().saturating_sub(1)), 0);
        self.page.stop_revealing_selection();
        self.page.settle_selection();
    }

    pub(crate) fn place_row(&mut self, row: ScreenRow) {
        let hidden_here = self.page.tabs.iter_mut().flat_map(|tabs| tabs.opened.iter_mut().flatten().flatten());
        let here = self.page.rows.iter_mut().chain(hidden_here);
        let arriving = self.fade.iter_mut().filter_map(|fade| match &mut fade.arrival {
            Some(Arrival::Page { page, .. }) => Some(page),
            _ => None,
        });
        let elsewhere = arriving.chain(self.navigation.visited.values_mut()).flat_map(|page| {
            let hidden = page.tabs.iter_mut().flat_map(|tabs| tabs.opened.iter_mut().flatten().flatten());
            page.rows.iter_mut().chain(hidden)
        });
        if let Some(slot) = here.chain(elsewhere).find(|slot| slot.pending && slot.id == row.id) {
            *slot = row;
        }
    }
}
