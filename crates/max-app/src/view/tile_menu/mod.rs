mod lines;

use eframe::egui::{self, Align2, Color32, CornerRadius, Event, Id, Key, Pos2, Rect, Sense, Stroke, Vec2};
pub use lines::{menu_lines, MenuLine, Pick};

use crate::{
    app::App,
    intent::{AccountChange, AccountChangeKind, Go, Intent, TileMenu, TileMenuRequest},
    metrics::{
        tile_menu::{LINE_HEIGHT, PROGRESS_WIDTH, WIDTH},
        BODY_TEXT,
    },
    model::ScreenTile,
    paint::{points_to_when_hovered, progress_bar, tick},
    theme::{self, bold, regular, PANEL, TEXT},
};

const CORNER: u8 = 8;
const VERTICAL_PADDING: f32 = 6.0;
const KEPT_INSIDE_WINDOW_BY: f32 = 8.0;
const LABEL_INSET: f32 = 18.0;
const ICON_COLUMN: f32 = 34.0;
const ICON_CENTER_FROM_LEFT: f32 = 28.0;
const ICON_TEXT: f32 = 18.0;
const SPACE_RIGHT_OF_LABEL: f32 = 56.0;
const LABEL_RAISED_OVER_PROGRESS: f32 = 5.0;
const PROGRESS_BELOW_LABEL: f32 = 4.0;
const PROGRESS_HEIGHT: f32 = 3.0;
const TICK_FROM_RIGHT: f32 = 24.0;
const TICK_SIZE: f32 = 6.5;
const OUTLINE: u8 = 30;
const HIGHLIGHT: u8 = 26;
const RULE: u8 = 28;
const PROGRESS_TRACK: u8 = 60;

impl App {
    pub(crate) fn draw_tile_menu(&self, ui: &mut egui::Ui, window: Rect, intent: &mut Intent) {
        let Some(menu) = self.tile_menu else { return };
        let Some(tile) = self.page.tile_at(menu.tile) else { return };
        // The page has already used up this frame's scroll amount, so scrolling is detected
        // from the events themselves.
        let clicked_away = ui.interact(window, Id::new("tile-menu-away"), Sense::click());
        let scrolled = ui.input(|input| input.events.iter().any(|event| matches!(event, Event::MouseWheel { .. })));
        if clicked_away.clicked() || clicked_away.secondary_clicked() || scrolled {
            intent.tile_menu = Some(TileMenuRequest::Close);
        }

        let lines = menu_lines(tile);
        let pressed = |key| ui.input(|input| input.key_pressed(key));
        let last = lines.len().saturating_sub(1);
        let moved_cursor = match (pressed(Key::ArrowDown), pressed(Key::ArrowUp)) {
            (true, _) => Some(menu.cursor.map_or(0, |cursor| (cursor + 1).min(last))),
            (_, true) => Some(menu.cursor.map_or(last, |cursor| cursor.saturating_sub(1))),
            _ => None,
        };
        if let Some(cursor) = moved_cursor.filter(|_| !lines.is_empty()) {
            intent.tile_menu = Some(TileMenuRequest::Open(TileMenu { cursor: Some(cursor), ..menu }));
        }
        let entered = pressed(Key::Enter);

        let panel = tile_menu_panel(ui, &lines, menu.position, window);
        let painter = ui.painter().clone();
        painter.rect_filled(panel.expand(1.0), CORNER + 1, Color32::from_white_alpha(OUTLINE));
        painter.rect_filled(panel, CORNER, PANEL);
        for (index, line) in lines.iter().enumerate() {
            let area =
                Rect::from_min_size(panel.min + Vec2::new(0.0, VERTICAL_PADDING + index as f32 * LINE_HEIGHT), Vec2::new(panel.width(), LINE_HEIGHT));
            let response = ui.interact(area, Id::new(("tile-menu", index)), Sense::click());
            points_to_when_hovered(ui, &response);
            if response.hovered() || menu.cursor == Some(index) {
                paint_highlight(&painter, panel, area, index == 0, index == last);
            }
            paint_menu_line(&painter, line, area);
            if response.clicked() || (entered && menu.cursor == Some(index)) {
                pick(line.pick, menu, tile, intent);
            }
        }
    }
}

fn tile_menu_panel(ui: &egui::Ui, lines: &[MenuLine], wanted_position: Pos2, window: Rect) -> Rect {
    let widest_label =
        lines.iter().map(|line| ui.painter().layout_no_wrap(line.label.to_string(), bold(BODY_TEXT), TEXT).size().x).fold(0.0, f32::max);
    let width = (widest_label + ICON_COLUMN + SPACE_RIGHT_OF_LABEL).max(WIDTH);
    let size = Vec2::new(width, lines.len() as f32 * LINE_HEIGHT + VERTICAL_PADDING * 2.0);
    let room = Rect::from_min_max(window.min + Vec2::splat(KEPT_INSIDE_WINDOW_BY), window.max - size - Vec2::splat(KEPT_INSIDE_WINDOW_BY));
    Rect::from_min_size(wanted_position.clamp(room.min, room.max.max(room.min)), size)
}

fn paint_menu_line(painter: &egui::Painter, line: &MenuLine, area: Rect) {
    if line.ruled_above {
        painter.line_segment([area.left_top(), area.right_top()], Stroke::new(1.0, Color32::from_white_alpha(RULE)));
    }
    let raised = if line.watched.is_some() { LABEL_RAISED_OVER_PROGRESS } else { 0.0 };
    let label_at = area.left_center() + Vec2::new(LABEL_INSET + ICON_COLUMN, -raised);
    let font = if line.chosen { bold(BODY_TEXT) } else { regular(BODY_TEXT) };
    let label = painter.text(label_at, Align2::LEFT_CENTER, line.label, font, TEXT);
    if let Some(watched) = line.watched {
        let track = Rect::from_min_size(Pos2::new(label.left(), label.bottom() + PROGRESS_BELOW_LABEL), Vec2::new(PROGRESS_WIDTH, PROGRESS_HEIGHT));
        progress_bar(painter, track, watched, PROGRESS_HEIGHT / 2.0, Color32::from_white_alpha(PROGRESS_TRACK), TEXT);
    }
    let icon_center = area.left_center() + Vec2::new(ICON_CENTER_FROM_LEFT, 0.0);
    painter.text(icon_center, Align2::CENTER_CENTER, line.icon, theme::icons(ICON_TEXT), TEXT);
    if line.chosen {
        tick(painter, area.right_center() - Vec2::new(TICK_FROM_RIGHT, 0.0), TICK_SIZE, Stroke::new(2.0, TEXT));
    }
}

fn paint_highlight(painter: &egui::Painter, panel: Rect, line: Rect, first: bool, last: bool) {
    let mut highlight = line;
    if first {
        highlight.min.y = panel.top();
    }
    if last {
        highlight.max.y = panel.bottom();
    }
    let (top, bottom) = (if first { CORNER } else { 0 }, if last { CORNER } else { 0 });
    painter.rect_filled(highlight, CornerRadius { nw: top, ne: top, sw: bottom, se: bottom }, Color32::from_white_alpha(HIGHLIGHT));
}

fn pick(pick: Pick, menu: TileMenu, tile: &ScreenTile, intent: &mut Intent) {
    let change = |kind| Some(AccountChange { tile: menu.tile, kind });
    match pick {
        Pick::OpenPage(route) => intent.go = Some(Go::Page(route.to_string())),
        Pick::Play(route) => intent.go = Some(Go::Play { route: route.to_string(), title: tile.title.clone() }),
        Pick::ToggleMyList => intent.account_change = change(AccountChangeKind::ToggleMyList),
        Pick::RemoveFromRow => intent.account_change = change(AccountChangeKind::RemoveFromRow),
        Pick::Rate(value) => intent.account_change = change(AccountChangeKind::Rate(value.to_string())),
    }
}
