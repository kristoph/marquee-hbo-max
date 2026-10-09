mod artwork;
mod buttons;
mod dots;
mod text;

use eframe::egui::{self, Align, Painter, Pos2, Rect, Sense, Vec2};

use crate::{
    app::App,
    intent::Intent,
    metrics::{
        hero::{BUTTONS_ABOVE_FIRST_RAIL, FRACTION_ABOVE_FIRST_RAIL},
        MARGIN,
    },
    model::{ScreenRow, ScreenTile},
};

const IN_VIEW_WHEN_FRACTION_VISIBLE: f32 = 0.5;
const ARTWORK_ASPECT: f32 = 16.0 / 9.0;
const FRACTION_OF_OVERFLOW_HIDDEN_ABOVE: f32 = 0.25;

/// The hero takes up the page only as far as the first rail, which is drawn over the foot of
/// its artwork: `flow` is the part it takes up and `bounds` all of it.
pub(super) struct HeroScene<'row> {
    pub row: &'row ScreenRow,
    pub tile: &'row ScreenTile,
    pub index: usize,
    pub flow: Rect,
    pub bounds: Rect,
    pub artwork: Rect,
    pub visible: Rect,
    pub painter: Painter,
    pub selected: bool,
}

impl HeroScene<'_> {
    pub fn left(&self) -> f32 {
        self.bounds.left() + MARGIN
    }

    pub fn button_top(&self) -> f32 {
        self.flow.bottom() - BUTTONS_ABOVE_FIRST_RAIL
    }
}

impl App {
    pub(crate) fn draw_hero(&self, ui: &mut egui::Ui, row: &ScreenRow, height: f32, intent: &mut Intent) {
        let width = ui.available_width();
        let (flow, _) = ui.allocate_exact_size(Vec2::new(width, height * FRACTION_ABOVE_FIRST_RAIL), Sense::hover());
        let bounds = Rect::from_min_size(flow.min, Vec2::new(width, height));
        let selected = self.page.selected.row == 0;
        if selected && self.page.is_revealing_selection() {
            ui.scroll_to_rect(flow, Some(Align::TOP));
        }
        if !ui.is_rect_visible(bounds) {
            return;
        }
        let index = self.hero.index.min(row.tiles.len() - 1);
        let visible = ui.clip_rect();
        let scene = HeroScene {
            row,
            tile: &row.tiles[index],
            index,
            flow,
            bounds,
            artwork: artwork_covering(bounds),
            visible,
            painter: ui.painter().with_clip_rect(visible.intersect(bounds)),
            selected,
        };
        self.hero.in_view.set(visible.intersect(flow).height() > flow.height() * IN_VIEW_WHEN_FRACTION_VISIBLE);

        self.paint_hero_artwork(ui, &scene);
        self.collect_hero_swipes(ui, &scene, intent);
        artwork::paint_shading(&scene);
        self.paint_hero_text(ui, &scene);
        self.draw_hero_buttons(ui, &scene, intent);
        self.draw_hero_dots(ui, &scene, intent);
    }
}

/// The artwork is as tall as the hero and, in a page wider than that makes it, as wide as the
/// page: it then runs past the hero's foot, more of it hidden below than above. In a page
/// narrower than the artwork its left is what goes out of sight.
fn artwork_covering(bounds: Rect) -> Rect {
    let width = bounds.width().max(bounds.height() * ARTWORK_ASPECT);
    let size = Vec2::new(width, width / ARTWORK_ASPECT);
    let hidden_above = (size.y - bounds.height()) * FRACTION_OF_OVERFLOW_HIDDEN_ABOVE;
    Rect::from_min_size(Pos2::new(bounds.right() - size.x, bounds.top() - hidden_above), size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hero(width: f32) -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 900.0))
    }

    #[test]
    fn the_artwork_fills_a_hero_of_its_own_shape() {
        assert_eq!(artwork_covering(hero(1600.0)), hero(1600.0));
    }

    #[test]
    fn a_narrower_page_hides_the_left_of_the_artwork() {
        assert_eq!(artwork_covering(hero(1000.0)), Rect::from_min_size(Pos2::new(-600.0, 0.0), Vec2::new(1600.0, 900.0)));
    }

    #[test]
    fn a_wider_page_is_covered_from_side_to_side() {
        let artwork = artwork_covering(hero(2560.0));
        assert_eq!((artwork.left(), artwork.width(), artwork.height()), (0.0, 2560.0, 1440.0));
        assert!(artwork.top() < 0.0 && artwork.bottom() > 900.0);
    }
}
