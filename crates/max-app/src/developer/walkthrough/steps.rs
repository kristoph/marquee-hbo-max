use std::time::Instant;

use super::Step;
use crate::{
    app::App,
    intent::{AccountChange, AccountChangeKind, Go, Intent, TileMenu, TileMenuRequest},
    model::{ScreenTile, Selection},
    view::tile_menu::{menu_lines, Pick},
};

const SECONDS_PER_LETTER: f32 = 0.13;
const SECONDS_BEFORE_GIVING_UP: f32 = 8.0;

#[derive(Clone, Copy)]
pub struct Attempt {
    pub seconds_overdue: f32,
    pub dry_run: bool,
    pub busy: bool,
}

impl Attempt {
    fn give_up(self) -> bool {
        self.seconds_overdue > SECONDS_BEFORE_GIVING_UP
    }
}

impl App {
    /// Returns whether the step is over; one that is not is tried again on the next frame.
    pub(super) fn perform(&mut self, step: Step, attempt: Attempt, intent: &mut Intent) -> bool {
        if matches!(step, Step::Back) && self.playback.is_open() {
            intent.go = Some(Go::Back);
            return true;
        }
        if attempt.busy || self.playback.is_open() {
            return false;
        }
        match step {
            Step::NextHeroTitle => self.step_hero(true),
            Step::DownRows(rows) => {
                let row = (0..rows).fold(self.page.selected.row, |row, _| self.page.row_after(row).unwrap_or(row));
                self.select_and_reveal(row, 0);
            }
            Step::AcrossToColumn(column) => self.select_and_reveal(self.page.selected.row, column),
            Step::TopOfPage => {
                let row = if self.page.rows[0].tiles.is_empty() { self.page.row_after(0).unwrap_or(0) } else { 0 };
                self.select_and_reveal(row, if self.page.hero_len().is_some() { self.hero.index } else { 0 });
            }
            Step::GoTo(route) => intent.go = Some(Go::Page(route.to_string())),
            Step::Type(text) => return self.type_into_search(text, attempt.seconds_overdue),
            Step::SelectTitle(title) => match self.find_title(title) {
                Some(found) => self.select_and_reveal(found.row, found.column),
                None => return attempt.give_up(),
            },
            Step::OpenTileMenu => match self.page.selected_artwork.get().filter(|_| !self.page.is_revealing_selection()) {
                Some(artwork) => intent.tile_menu = Some(TileMenuRequest::Open(TileMenu::beside_artwork(self.page.selected, artwork))),
                None => return attempt.give_up(),
            },
            Step::MoveToMyListLine => {
                if let Some(menu) = self.tile_menu {
                    let cursor =
                        self.page.tile_at(menu.tile).and_then(|tile| menu_lines(tile).iter().position(|line| line.pick == Pick::ToggleMyList));
                    intent.tile_menu = cursor.map(|cursor| TileMenuRequest::Open(TileMenu { cursor: Some(cursor), ..menu }));
                }
            }
            Step::AddToMyList => self.add_to_my_list(attempt.dry_run, intent),
            Step::CloseTileMenu => intent.tile_menu = Some(TileMenuRequest::Close),
            Step::Play if !attempt.dry_run => intent.go = self.selected_destination(),
            Step::Back if !attempt.dry_run => intent.go = Some(Go::Back),
            Step::Play | Step::Back => {}
        }
        true
    }

    fn type_into_search(&mut self, text: &str, seconds_typing: f32) -> bool {
        let Some(search) = &mut self.page.search else { return false };
        let letters = ((seconds_typing / SECONDS_PER_LETTER) as usize + 1).min(text.chars().count());
        let typed: String = text.chars().take(letters).collect();
        if typed != search.text {
            search.text = typed;
            search.edited = Some(Instant::now());
        }
        letters == text.chars().count()
    }

    fn find_title(&self, title: &str) -> Option<Selection> {
        let tiles = || {
            self.page
                .rows
                .iter()
                .enumerate()
                .flat_map(|(row, tiles)| tiles.tiles.iter().enumerate().map(move |(column, tile)| (Selection::new(row, column), tile)))
        };
        let exact = tiles().find(|(_, tile)| tile.title.eq_ignore_ascii_case(title));
        let starting_with = || tiles().find(|(_, tile)| tile.title.to_lowercase().starts_with(&title.to_lowercase()));
        exact.or_else(starting_with).map(|(at, _)| at)
    }

    fn add_to_my_list(&mut self, dry_run: bool, intent: &mut Intent) {
        let Some(menu) = self.tile_menu else { return };
        let already_listed = self.page.tile_at(menu.tile).and_then(ScreenTile::is_on_my_list).unwrap_or(false);
        if dry_run || already_listed {
            intent.tile_menu = Some(TileMenuRequest::Close);
        } else {
            intent.account_change = Some(AccountChange { tile: menu.tile, kind: AccountChangeKind::ToggleMyList });
        }
    }
}
