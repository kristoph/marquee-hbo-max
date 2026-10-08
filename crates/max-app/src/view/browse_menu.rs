use eframe::egui::{self, Align2, Color32, Id, Pos2, Rect, Sense, Vec2};

use crate::{
    app::App,
    intent::{Go, Intent},
    metrics::{
        header::{BROWSE_MENU_LINE, BROWSE_MENU_WIDTH, HEIGHT},
        MARGIN,
    },
    paint::points_to_when_hovered,
    theme::{bold, regular, BROWSE_MENU, DIM_TEXT, TEXT},
};

const SCRIM: u8 = 150;
const HOVERED_LINE: u8 = 22;
const SPACE_ABOVE_LINES: f32 = 12.0;
const LINE_TEXT: f32 = 20.0;
const ABOUT: &str = "About";
const ABOUT_TEXT: f32 = 17.0;

impl App {
    pub(crate) fn draw_browse_menu(&self, ui: &mut egui::Ui, window: Rect, intent: &mut Intent) {
        if ui.interact(window, Id::new("menu-scrim"), Sense::click()).clicked() {
            intent.browse_menu = Some(false);
        }
        let painter = ui.painter().clone();
        painter.rect_filled(window, 0.0, Color32::from_black_alpha(SCRIM));
        let top = window.top() + HEIGHT;
        let panel = Rect::from_min_size(Pos2::new(window.left(), top), Vec2::new(BROWSE_MENU_WIDTH, window.bottom() - top));
        painter.rect_filled(panel, 0.0, BROWSE_MENU);

        for (index, item) in self.chrome.navigation.iter().enumerate() {
            let corner = Pos2::new(panel.left(), panel.top() + SPACE_ABOVE_LINES + index as f32 * BROWSE_MENU_LINE);
            let line = Rect::from_min_size(corner, Vec2::new(panel.width(), BROWSE_MENU_LINE));
            let response = ui.interact(line, Id::new(("menu-item", index)), Sense::click());
            points_to_when_hovered(ui, &response);
            if response.hovered() {
                painter.rect_filled(line, 0.0, Color32::from_white_alpha(HOVERED_LINE));
            }
            let is_current_page = item.route.as_deref() == Some(self.navigation.route.as_str());
            let font = if is_current_page || response.hovered() { bold(LINE_TEXT) } else { regular(LINE_TEXT) };
            let ink = if item.route.is_some() { TEXT } else { Color32::GRAY };
            painter.text(line.left_center() + Vec2::new(MARGIN, 0.0), Align2::LEFT_CENTER, &item.label, font, ink);
            if response.clicked() {
                match &item.route {
                    Some(route) => intent.go = Some(Go::Page(route.clone())),
                    None => intent.browse_menu = Some(false),
                }
            }
        }
        draw_about_line(ui, panel, intent);
    }
}

fn draw_about_line(ui: &mut egui::Ui, panel: Rect, intent: &mut Intent) {
    let line = Rect::from_min_max(
        Pos2::new(panel.left(), panel.bottom() - BROWSE_MENU_LINE - SPACE_ABOVE_LINES),
        panel.max - Vec2::new(0.0, SPACE_ABOVE_LINES),
    );
    let response = ui.interact(line, Id::new("menu-about"), Sense::click());
    points_to_when_hovered(ui, &response);
    if response.hovered() {
        ui.painter().rect_filled(line, 0.0, Color32::from_white_alpha(HOVERED_LINE));
    }
    let ink = if response.hovered() { TEXT } else { DIM_TEXT };
    ui.painter().text(line.left_center() + Vec2::new(MARGIN, 0.0), Align2::LEFT_CENTER, ABOUT, regular(ABOUT_TEXT), ink);
    if response.clicked() {
        intent.about = Some(true);
    }
}
