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
        let artwork_width = height * 16.0 / 9.0;
        let index = self.hero.index.min(row.tiles.len() - 1);
        let visible = ui.clip_rect();
        let scene = HeroScene {
            row,
            tile: &row.tiles[index],
            index,
            flow,
            bounds,
            artwork: Rect::from_min_size(Pos2::new(bounds.right() - artwork_width, bounds.top()), Vec2::new(artwork_width, height)),
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
