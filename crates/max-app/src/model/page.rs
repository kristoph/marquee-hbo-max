use max_api::{
    client::SEARCH_RESULTS_ALIAS,
    cms::{Filter, Page, Row},
};

use super::{
    builder::{Outstanding, ScreenBuilder},
    lazy_image::LazyImage,
    rail::ScreenRow,
    selection::Selection,
};

const ROWS_ON_THE_FIRST_SCREEN: usize = 3;
const TILES_ACROSS_THE_FIRST_SCREEN: usize = 8;

#[derive(Clone)]
pub struct SearchSource {
    pub results_id: String,
    pub topics: Vec<Filter>,
}

/// The rows of the selected tab sit in the page's row list from `first_row`, between the rows
/// before the tab group and the rows after it.
pub struct ScreenTabs {
    pub first_row: usize,
    pub titles: Vec<String>,
    pub selected: usize,
    pub rows_shown: usize,
    pub opened: Vec<Option<Vec<ScreenRow>>>,
    pub unopened: Vec<Option<Vec<Row>>>,
}

pub struct ScreenPage {
    pub rows: Vec<ScreenRow>,
    pub tabs: Option<ScreenTabs>,
    pub selected: Selection,
    pub scroll: f32,
    pub search: Option<SearchSource>,
}

impl ScreenPage {
    pub fn first_screen_ready(&self) -> bool {
        self.rows.iter().filter(|row| row.takes_space()).take(ROWS_ON_THE_FIRST_SCREEN).all(|row| {
            !row.pending
                && row.masthead.as_ref().is_none_or(|masthead| masthead.image.is_ready())
                && row
                    .tiles
                    .iter()
                    .take(TILES_ACROSS_THE_FIRST_SCREEN)
                    .all(|tile| tile.artwork.as_ref().is_none_or(LazyImage::is_ready) && tile.logo.as_ref().is_none_or(|logo| logo.image.is_ready()))
        })
    }
}

pub fn build_page(mut page: Page, fetch_deferred_rows: bool) -> (ScreenPage, Outstanding) {
    let mut builder = ScreenBuilder::new(fetch_deferred_rows);
    let search = page
        .rows
        .iter()
        .find(|row| row.alias == SEARCH_RESULTS_ALIAS)
        .map(|row| SearchSource { results_id: row.id.clone(), topics: row.filters.clone() });
    let Some(group_index) = page.rows.iter().position(|row| !row.tabs.is_empty()) else {
        let rows = builder.rows(&page.rows);
        return (ScreenPage { rows, tabs: None, selected: Selection::FIRST, scroll: 0.0, search }, builder.outstanding);
    };

    let rows_after_group = page.rows.split_off(group_index + 1);
    let group = page.rows.pop().expect("the tab group is the last row left");
    let mut rows = builder.rows(&page.rows);
    let first_row = rows.len();
    let titles = group.tabs.iter().map(|tab| tab.title.clone()).collect();
    let mut unopened: Vec<Option<Vec<Row>>> = group.tabs.into_iter().map(|tab| Some(tab.rows)).collect();
    let first_tab = builder.rows(&unopened[0].take().unwrap_or_default());
    let mut opened: Vec<Option<Vec<ScreenRow>>> = unopened.iter().map(|_| None).collect();
    opened[0] = Some(Vec::new());
    let rows_shown = first_tab.len();
    rows.extend(first_tab);
    rows.extend(builder.rows(&rows_after_group));
    let tabs = ScreenTabs { first_row, titles, selected: 0, rows_shown, opened, unopened };
    (ScreenPage { rows, tabs: Some(tabs), selected: Selection::FIRST, scroll: 0.0, search }, builder.outstanding)
}
