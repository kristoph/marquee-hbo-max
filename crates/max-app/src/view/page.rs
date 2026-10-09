use eframe::egui::{self, scroll_area::ScrollBarVisibility, Rect, ScrollArea, UiBuilder, Vec2};

use super::{search_bar, topics};
use crate::{
    app::App,
    intent::Intent,
    metrics::{hero, MAX_CONTENT_WIDTH, SPACE_ABOVE_FIRST_RAIL, SPACE_BELOW_PAGE},
};

const HEADER_TURNS_SOLID_AFTER_SCROLLING: f32 = 8.0;

impl App {
    pub(crate) fn draw_page(&mut self, ui: &mut egui::Ui, intent: &mut Intent) {
        let window = ui.max_rect();
        let content = centred_content(window);
        self.lay_grid_out_again(content.width());
        self.page.width = content.width();
        self.page.selected_artwork.set(None);
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
        // The header and the menus span the window; only the page itself is kept to a width.
        let mut page_ui = ui.new_child(UiBuilder::new().max_rect(content));
        page_ui.set_clip_rect(content);
        let page = area.show(&mut page_ui, |ui| {
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
                    self.draw_hero(ui, row, hero::FULL_HEIGHT, intent);
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

fn centred_content(window: Rect) -> Rect {
    Rect::from_center_size(window.center(), Vec2::new(window.width().min(MAX_CONTENT_WIDTH), window.height()))
}

#[cfg(test)]
mod tests {
    use eframe::egui::Pos2;

    use super::*;

    #[test]
    fn a_window_wider_than_the_page_shows_it_centred_and_a_narrower_one_in_full() {
        let wide = Rect::from_min_size(Pos2::ZERO, Vec2::new(MAX_CONTENT_WIDTH + 640.0, 1000.0));
        assert_eq!(centred_content(wide), Rect::from_min_size(Pos2::new(320.0, 0.0), Vec2::new(MAX_CONTENT_WIDTH, 1000.0)));
        let narrow = Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 1000.0));
        assert_eq!(centred_content(narrow), narrow);
    }
}
