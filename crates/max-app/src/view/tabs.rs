use eframe::egui::{self, scroll_area::ScrollBarVisibility, Color32, ScrollArea, Sense, Vec2};

use crate::{
    app::App,
    intent::Intent,
    metrics::{BODY_TEXT, MARGIN},
    paint::points_to_when_hovered,
    theme::{bold, TEXT, TRANSLUCENT},
};

const PILL_HEIGHT: f32 = 40.0;
const PILL_PADDING: f32 = 36.0;
const PILL_GAP: f32 = 10.0;
const SPACE_BELOW: f32 = 24.0;
const HOVERED_PILL: u8 = 70;

impl App {
    pub(crate) fn draw_tabs(&self, ui: &mut egui::Ui, intent: &mut Intent) {
        let Some(tabs) = &self.page.tabs else { return };
        ScrollArea::horizontal()
            .id_salt(("tabs", self.navigation.generation))
            .auto_shrink([false, true])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = PILL_GAP;
                    ui.add_space(MARGIN);
                    for (index, title) in tabs.titles.iter().enumerate() {
                        let selected = index == tabs.selected;
                        let ink = if selected { Color32::BLACK } else { TEXT };
                        let label = ui.painter().layout_no_wrap(title.clone(), bold(BODY_TEXT), ink);
                        let (pill, response) = ui.allocate_exact_size(Vec2::new(label.size().x + PILL_PADDING, PILL_HEIGHT), Sense::click());
                        points_to_when_hovered(ui, &response);
                        let fill = match (selected, response.hovered()) {
                            (true, _) => Color32::WHITE,
                            (false, true) => Color32::from_white_alpha(HOVERED_PILL),
                            (false, false) => TRANSLUCENT,
                        };
                        ui.painter().rect_filled(pill, PILL_HEIGHT / 2.0, fill);
                        ui.painter().galley(pill.center() - label.size() / 2.0, label, ink);
                        if response.clicked() {
                            intent.tab = Some(index);
                        }
                    }
                    ui.add_space(MARGIN);
                });
            });
        ui.add_space(SPACE_BELOW);
    }
}
