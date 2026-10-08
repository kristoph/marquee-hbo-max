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
mod tests {
    use max_api::cms::{Layout, TileDetail};

    use super::*;

    fn tile(title: &str) -> ScreenTile {
        ScreenTile {
            title: title.to_string(),
            route: None,
            artwork: None,
            logo: None,
            badge_icon: None,
            banner_icon: None,
            detail: TileDetail::default(),
        }
    }

    fn page(tiles_per_row: &[usize]) -> OpenPage {
        let mut page = OpenPage::empty(Selection::FIRST);
        page.rows = tiles_per_row
            .iter()
            .enumerate()
            .map(|(row, count)| ScreenRow::untitled(row.to_string(), Layout::Poster, (0..*count).map(|column| tile(&column.to_string())).collect()))
            .collect();
        page
    }

    #[test]
    fn moving_between_rows_passes_over_empty_ones() {
        let page = page(&[3, 0, 0, 2]);
        assert_eq!((page.row_after(0), page.row_before(3)), (Some(3), Some(0)));
        assert_eq!((page.row_after(3), page.row_before(0)), (None, None));
    }

    #[test]
    fn selecting_keeps_the_column_within_the_row() {
        let mut page = page(&[5, 2]);
        page.select(1, 4);
        assert_eq!(page.selected, Selection::new(1, 1));
        assert_eq!(page.tile_at(page.selected).unwrap().title, "1");
    }

    #[test]
    fn a_selection_left_on_an_emptied_row_settles_on_the_next_row_with_tiles() {
        let mut page = page(&[4, 0, 3]);
        page.selected = Selection::new(1, 3);
        page.settle_selection();
        assert_eq!(page.selected, Selection::new(2, 2));
    }

    #[test]
    fn revealing_the_selection_lasts_a_few_frames() {
        let mut page = page(&[1]);
        page.reveal_selection();
        for _ in 0..FRAMES_TO_REVEAL_SELECTION {
            assert!(page.is_revealing_selection());
            page.selection_was_revealed_this_frame();
        }
        assert!(!page.is_revealing_selection());
    }
}
