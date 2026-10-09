use max_api::cms::Layout;

use super::*;

fn page(tiles_per_row: &[usize]) -> OpenPage {
    let mut page = OpenPage::empty(Selection::FIRST);
    page.rows = tiles_per_row
        .iter()
        .enumerate()
        .map(|(row, count)| {
            ScreenRow::untitled(row.to_string(), Layout::Poster, (0..*count).map(|column| ScreenTile::titled(&column.to_string())).collect())
        })
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
