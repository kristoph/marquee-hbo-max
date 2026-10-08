use eframe::egui::{self, Align2, Color32, Id, Image, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2};

use super::{caption, geometry::RailGeometry, overlays};
use crate::{
    app::App,
    intent::{Go, Intent, TileMenu, TileMenuRequest},
    metrics::{
        rail::{MORE_BUTTON, SELECTED_SCALE, SELECTION_RING_RADIUS, TILE_CORNER_RADIUS},
        top_ten, BODY_TEXT,
    },
    model::{ScreenRow, ScreenTile, Selection},
    paint::{points_to_when_hovered, WrappedText},
    theme::{self, bold, PLACEHOLDER, RANK_NUMERAL, TEXT},
    timing::{ARTWORK_FADE_SECONDS, SELECTION_GROW_SECONDS},
};

const SELECTION_RING_WIDTH: f32 = 3.0;
const TITLE_INSET_WITHOUT_ARTWORK: f32 = 16.0;
const NUMERAL_TEXT: f32 = 90.0;
const MORE_BUTTON_INSET: f32 = 8.0;
const MORE_BUTTON_DOT_SPACING: f32 = 6.0;
const MORE_BUTTON_DOT_RADIUS: f32 = 1.8;
const TILE_MENU_BELOW_MORE_BUTTON: f32 = 6.0;

#[derive(Clone, Copy)]
pub(super) struct TileCell<'row> {
    pub tile: &'row ScreenTile,
    pub at: Selection,
    pub bounds: Rect,
    pub response: &'row Response,
}

/// `base` is where the artwork rests and `artwork` where it is drawn, larger while selected.
pub(super) struct TileScene<'row> {
    pub row: &'row ScreenRow,
    pub tile: &'row ScreenTile,
    pub at: Selection,
    pub base: Rect,
    pub artwork: Rect,
    pub selected: bool,
}

impl App {
    pub(super) fn collect_tile_clicks(
        &self,
        response: &Response,
        cell: Rect,
        tile: &ScreenTile,
        at: Selection,
        pointer_moved: bool,
        intent: &mut Intent,
    ) {
        if response.clicked() {
            intent.select = Some(at);
            intent.go = tile.route.as_ref().map(|route| Go::to(route, &tile.title));
        } else if response.secondary_clicked() {
            intent.select = Some(at);
            let position = response.interact_pointer_pos().unwrap_or(cell.center());
            intent.tile_menu = Some(TileMenuRequest::Open(TileMenu::at(at, position)));
        } else if response.hovered() && pointer_moved && !self.browse_menu_open && self.tile_menu.is_none() {
            intent.select = Some(at);
        }
    }

    pub(super) fn draw_tile(&self, ui: &mut egui::Ui, row: &ScreenRow, geometry: &RailGeometry, cell: TileCell, intent: &mut Intent) {
        let TileCell { tile, at, bounds, response } = cell;
        let selected = self.page.selected == at;
        let growth = ui.ctx().animate_bool_with_time(response.id.with("focus"), selected, SELECTION_GROW_SECONDS);
        let base = Rect::from_min_size(bounds.min + Vec2::new(geometry.padding + geometry.numeral_room, geometry.padding), geometry.artwork);
        let artwork = Rect::from_center_size(base.center(), geometry.artwork * (1.0 + (SELECTED_SCALE - 1.0) * growth));
        let scene = TileScene { row, tile, at, base, artwork, selected };

        if row.numbered {
            paint_rank_numeral(ui, &scene);
        }
        self.paint_tile_artwork(ui, &scene);
        overlays::paint(ui, &scene);
        if selected {
            self.draw_selection(ui, &scene, intent);
        }
        if geometry.captioned {
            caption::paint(ui.painter(), &scene);
        }
    }

    fn paint_tile_artwork(&self, ui: &mut egui::Ui, scene: &TileScene) {
        let Some(uri) = scene.tile.artwork.as_ref().and_then(|artwork| artwork.uri()) else {
            ui.painter().rect_filled(scene.artwork, TILE_CORNER_RADIUS, PLACEHOLDER);
            let inner = scene.artwork.shrink(TITLE_INSET_WITHOUT_ARTWORK);
            let title = WrappedText { text: &scene.tile.title, font: bold(BODY_TEXT), color: TEXT, width: inner.width(), lines: 3 };
            title.paint(ui.painter(), inner.left_top());
            return;
        };
        let image = Image::new(uri).corner_radius(TILE_CORNER_RADIUS).show_loading_spinner(false);
        let ready = matches!(image.load_for_size(ui.ctx(), scene.artwork.size()), Ok(egui::load::TexturePoll::Ready { .. }));
        let opacity = ui.ctx().animate_bool_with_time(Id::new(("artwork", uri)), ready, ARTWORK_FADE_SECONDS);
        if opacity < 1.0 {
            ui.painter().rect_filled(scene.artwork, TILE_CORNER_RADIUS, PLACEHOLDER);
        }
        match ready {
            true => image.tint(theme::white(opacity)).paint_at(ui, scene.artwork),
            false => self.developer.tiles_drawn_before_their_artwork.set(self.developer.tiles_drawn_before_their_artwork.get() + 1),
        }
    }

    fn draw_selection(&self, ui: &mut egui::Ui, scene: &TileScene, intent: &mut Intent) {
        let ring = Stroke::new(SELECTION_RING_WIDTH, TEXT);
        ui.painter().rect_stroke(scene.artwork.expand(SELECTION_RING_WIDTH), SELECTION_RING_RADIUS, ring, StrokeKind::Outside);
        self.page.selected_artwork.set(Some(scene.artwork));
        if scene.tile.detail.menu.is_empty() {
            return;
        }
        let corner = scene.artwork.right_top() + Vec2::new(-MORE_BUTTON - MORE_BUTTON_INSET, MORE_BUTTON_INSET);
        let button = Rect::from_min_size(corner, Vec2::splat(MORE_BUTTON));
        let response = ui.interact(button, Id::new(("more", self.navigation.generation, scene.at.row, scene.at.column)), Sense::click());
        points_to_when_hovered(ui, &response);
        let painter = ui.painter();
        painter.circle_filled(button.center(), MORE_BUTTON / 2.0, Color32::from_black_alpha(if response.hovered() { 230 } else { 170 }));
        for dot in [-1.0, 0.0, 1.0] {
            painter.circle_filled(button.center() + Vec2::new(0.0, dot * MORE_BUTTON_DOT_SPACING), MORE_BUTTON_DOT_RADIUS, TEXT);
        }
        if response.clicked() {
            intent.go = None;
            let below = button.left_bottom() + Vec2::new(0.0, TILE_MENU_BELOW_MORE_BUTTON);
            intent.tile_menu = Some(TileMenuRequest::Open(TileMenu::at(scene.at, below)));
        }
    }
}

fn paint_rank_numeral(ui: &mut egui::Ui, scene: &TileScene) {
    let corner =
        Pos2::new(scene.base.left() - top_ten::NUMERAL_LEFT_OF_POSTER, scene.base.bottom() - top_ten::NUMERAL.y - top_ten::NUMERAL_ABOVE_POSTER_FOOT);
    let area = Rect::from_min_size(corner, top_ten::NUMERAL);
    let image = scene.row.rank_numerals.get(scene.at.column).and_then(|numeral| {
        let when_selected = numeral.selected.as_ref().filter(|_| scene.selected).and_then(|image| image.uri());
        when_selected.or(numeral.plain.as_ref()?.uri())
    });
    match image {
        Some(uri) => {
            Image::new(uri).paint_at(ui, area);
        }
        None => {
            let digit = (scene.at.column + 1).to_string();
            ui.painter().text(area.center(), Align2::CENTER_CENTER, digit, bold(NUMERAL_TEXT), RANK_NUMERAL);
        }
    }
}
