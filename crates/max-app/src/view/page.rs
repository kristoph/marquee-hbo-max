use eframe::egui::{self, scroll_area::ScrollBarVisibility, ScrollArea, Vec2};

use super::{search_bar, topics};
use crate::{
    app::App,
    intent::Intent,
    metrics::{hero, SPACE_ABOVE_FIRST_RAIL, SPACE_BELOW_PAGE},
};

const HEADER_TURNS_SOLID_AFTER_SCROLLING: f32 = 8.0;
const LARGEST_FRACTION_OF_WINDOW_FOR_HERO: f32 = 0.96;

impl App {
    pub(crate) fn draw_page(&mut self, ui: &mut egui::Ui, intent: &mut Intent) {
        let window = ui.max_rect();
        self.lay_grid_out_again(window.width());
        self.page.width = window.width();
        self.page.selected_artwork.set(None);
        let hero_height = hero::FULL_HEIGHT.min(window.width() * 9.0 / 16.0).min(window.height() * LARGEST_FRACTION_OF_WINDOW_FOR_HERO);
        let pointer_moved = ui.input(|input| input.pointer.delta() != Vec2::ZERO);

        // The page is drawn through `&self`, so the search state it edits is lent out for the frame.
        let mut search = self.page.search.take();
        let mut area = ScrollArea::vertical()
            .id_salt(("page", self.navigation.generation))
            .auto_shrink(false)
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden);
        if let Some(offset) = self.page.scroll_to.take() {
            area = area.vertical_scroll_offset(offset);
        }
        let page = area.show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let tabs_before_row = self.page.tabs.as_ref().map(|tabs| tabs.first_row);
            let mut nothing_drawn_yet = true;
            if let Some(search) = search.as_mut() {
                search_bar::draw(ui, search);
                topics::draw(ui, search);
                nothing_drawn_yet = false;
            }
            for (row_index, row) in self.page.rows.iter().enumerate() {
                if tabs_before_row == Some(row_index) {
                    if std::mem::take(&mut nothing_drawn_yet) {
                        ui.add_space(SPACE_ABOVE_FIRST_RAIL);
                    }
                    self.draw_tabs(ui, intent);
                }
                if !row.takes_space() {
                    continue;
                }
                let leads_the_page = std::mem::take(&mut nothing_drawn_yet);
                if leads_the_page && row.is_hero() {
                    self.draw_hero(ui, row, hero_height, intent);
                    continue;
                }
                if leads_the_page {
                    ui.add_space(SPACE_ABOVE_FIRST_RAIL);
                }
                self.draw_rail(ui, row_index, row, pointer_moved, intent);
            }
            if tabs_before_row == Some(self.page.rows.len()) {
                self.draw_tabs(ui, intent);
            }
            ui.add_space(SPACE_BELOW_PAGE);
        });
        self.page.search = search;
        self.page.scroll = page.state.offset.y;

        if self.browse_menu_open {
            self.draw_browse_menu(ui, window, intent);
        }
        let solid_header = self.page.scroll > HEADER_TURNS_SOLID_AFTER_SCROLLING || self.browse_menu_open;
        self.draw_header(ui, window, solid_header, intent);
        self.draw_tile_menu(ui, window, intent);
        if self.about_open {
            super::about::draw(ui, window, intent);
        }
        self.draw_toast(ui, window);

        let selected_row_still_loading = self.page.rows.get(self.page.selected.row).is_some_and(|row| row.pending);
        if self.page.is_revealing_selection() && !selected_row_still_loading {
            self.page.selection_was_revealed_this_frame();
            ui.ctx().request_repaint();
        }
    }
}
