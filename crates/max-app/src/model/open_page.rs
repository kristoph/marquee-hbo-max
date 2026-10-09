use std::cell::Cell;

use eframe::egui::Rect;

use super::{
    page::ScreenTabs,
    rail::{ScreenRow, ScreenTile},
    search::Search,
    selection::Selection,
};

/// The toolkit may discard the frame in which a scroll request is first made, so the request
/// is repeated for a few frames.
const FRAMES_TO_REVEAL_SELECTION: u8 = 3;
const WIDTH_BEFORE_FIRST_FRAME: f32 = 1600.0;

/// The page on screen and where the person is on it.
pub struct OpenPage {
    pub rows: Vec<ScreenRow>,
    pub tabs: Option<ScreenTabs>,
    pub search: Option<Search>,
    pub width: f32,
    pub scroll: f32,
    pub scroll_to: Option<f32>,
    pub selected: Selection,
    pub selected_artwork: Cell<Option<Rect>>,
    frames_left_to_reveal_selection: u8,
}

impl OpenPage {
    pub fn empty(selected: Selection) -> Self {
        let mut page = Self {
            rows: Vec::new(),
            tabs: None,
            search: None,
            width: WIDTH_BEFORE_FIRST_FRAME,
            scroll: 0.0,
            scroll_to: None,
            selected,
            selected_artwork: Cell::new(None),
            frames_left_to_reveal_selection: 0,
        };
        if selected != Selection::FIRST {
            page.reveal_selection();
        }
        page
    }

    pub fn clear(&mut self) {
        *self = Self { width: self.width, ..Self::empty(Selection::FIRST) };
    }

    pub fn tile_at(&self, at: Selection) -> Option<&ScreenTile> {
        self.rows.get(at.row)?.tiles.get(at.column)
    }

    pub fn row_after(&self, row: usize) -> Option<usize> {
        (row + 1..self.rows.len()).find(|&candidate| !self.rows[candidate].tiles.is_empty())
    }

    pub fn row_before(&self, row: usize) -> Option<usize> {
        (0..row.min(self.rows.len())).rev().find(|&candidate| !self.rows[candidate].tiles.is_empty())
    }

    pub fn last_column(&self, row: usize) -> usize {
        self.rows.get(row).map_or(0, |row| row.tiles.len().saturating_sub(1))
    }

    pub fn hero(&self) -> Option<&ScreenRow> {
        self.rows.first().filter(|row| row.is_hero())
    }

    pub fn hero_len(&self) -> Option<usize> {
        self.hero().map(|hero| hero.tiles.len())
    }

    pub fn select(&mut self, row: usize, column: usize) {
        self.selected = Selection::new(row, column.min(self.last_column(row)));
    }

    /// Moves the selection onto a row that has tiles, after the rows have changed under it.
    pub fn settle_selection(&mut self) {
        let Selection { row, column } = self.selected;
        if self.rows.get(row).is_some_and(|row| !row.tiles.is_empty()) {
            return;
        }
        if let Some(row) = self.row_after(row.saturating_sub(1)).or_else(|| self.row_before(row)) {
            self.select(row, column);
        }
    }

    pub fn reveal_selection(&mut self) {
        self.frames_left_to_reveal_selection = FRAMES_TO_REVEAL_SELECTION;
    }

    pub fn stop_revealing_selection(&mut self) {
        self.frames_left_to_reveal_selection = 0;
    }

    pub fn is_revealing_selection(&self) -> bool {
        self.frames_left_to_reveal_selection > 0
    }

    pub fn selection_was_revealed_this_frame(&mut self) {
        self.frames_left_to_reveal_selection = self.frames_left_to_reveal_selection.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests;
