use eframe::egui::{self, Key};

use super::App;
use crate::{
    intent::{Go, Intent, TileMenu, TileMenuRequest},
    model::Selection,
};

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

const ARROW_KEYS: [(Key, Direction); 4] =
    [(Key::ArrowDown, Direction::Down), (Key::ArrowUp, Direction::Up), (Key::ArrowRight, Direction::Right), (Key::ArrowLeft, Direction::Left)];

impl App {
    pub(crate) fn handle_keys(&mut self, ctx: &egui::Context, intent: &mut Intent) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let pressed = |key| ctx.input(|input| input.key_pressed(key));
        if pressed(Key::Escape) || pressed(Key::Backspace) {
            intent.go = Some(Go::Back);
        }
        let page_has_the_keys = !self.playback.is_open() && self.profile_picker.is_none() && !self.browse_menu_open && self.tile_menu.is_none();
        if self.page.rows.is_empty() || !page_has_the_keys {
            return;
        }
        if pressed(Key::P) {
            self.switch_player();
        }
        if pressed(Key::M) {
            intent.tile_menu =
                self.page.selected_artwork.get().map(|artwork| TileMenuRequest::Open(TileMenu::beside_artwork(self.page.selected, artwork)));
        }
        if pressed(Key::Enter) || pressed(Key::Space) {
            intent.go = self.selected_destination();
        }
        let directions: Vec<Direction> = ARROW_KEYS.iter().filter(|(key, _)| pressed(*key)).map(|(_, direction)| *direction).collect();
        self.move_selection(&directions);
    }

    fn move_selection(&mut self, directions: &[Direction]) {
        let Selection { mut row, mut column } = self.page.selected;
        for direction in directions {
            match direction {
                Direction::Down => row = self.page.row_after(row).unwrap_or(row),
                Direction::Up => row = self.page.row_before(row).unwrap_or(row),
                Direction::Right => column += 1,
                Direction::Left => column = column.saturating_sub(1),
            }
        }
        row = row.min(self.page.rows.len() - 1);
        let onto_hero = row == 0 && self.page.hero_len().is_some();
        if onto_hero && self.page.selected.row != 0 {
            column = self.hero.index;
        }
        let moved = Selection::new(row, column.min(self.page.last_column(row)));
        if moved != self.page.selected {
            self.page.selected = moved;
            self.page.reveal_selection();
            if onto_hero {
                self.set_hero(moved.column);
            }
        }
    }
}
