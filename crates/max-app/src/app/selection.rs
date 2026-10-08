use super::App;
use crate::intent::Go;

impl App {
    pub(crate) fn select_and_reveal(&mut self, row: usize, column: usize) {
        self.page.select(row, column);
        self.page.reveal_selection();
        if row == 0 && self.page.hero_len().is_some() {
            self.set_hero(self.page.selected.column);
        }
    }

    /// Activating a hero title plays it; activating any other tile follows its own route.
    pub(crate) fn selected_destination(&self) -> Option<Go> {
        let row = self.page.rows.get(self.page.selected.row)?;
        let tile = row.tiles.get(self.page.selected.column)?;
        let play_route = if row.is_hero() { tile.play_action().and_then(|action| action.route.as_ref()) } else { None };
        Some(Go::to(play_route.or(tile.route.as_ref())?, &tile.title))
    }
}
