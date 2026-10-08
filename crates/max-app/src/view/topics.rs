use eframe::egui::{self, scroll_area::ScrollBarVisibility, Color32, ScrollArea, Sense, Vec2};

use crate::{
    metrics::{
        search::{BAR_TO_TOPICS, TOPICS_TO_ROWS, TOPIC_HEIGHT},
        BODY_TEXT, MARGIN,
    },
    model::Search,
    paint::points_to_when_hovered,
    theme::{bold, regular, DIM_TEXT, TEXT},
};

const NO_TOPIC_LABEL: &str = "Recommended";
const PILL_PADDING: f32 = 64.0;
const PILL_GAP: f32 = 14.0;
const SELECTED_PILL: Color32 = Color32::from_rgb(74, 74, 74);
const HOVERED_PILL: u8 = 60;
const RESTING_PILL: u8 = 33;

pub(crate) fn draw(ui: &mut egui::Ui, search: &mut Search) {
    ui.add_space(BAR_TO_TOPICS);
    let no_topic = search.text.is_empty().then_some((None, NO_TOPIC_LABEL));
    let mut picked = None;
    ScrollArea::horizontal().id_salt("search-topics").auto_shrink([false, true]).scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden).show(
        ui,
        |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = PILL_GAP;
                ui.add_space(MARGIN);
                let topics = search.topics.iter().enumerate().map(|(index, topic)| (Some(index), topic.label.as_str()));
                for (index, label) in no_topic.into_iter().chain(topics) {
                    let selected = index == search.topic;
                    let (font, ink) = if selected { (bold(BODY_TEXT), TEXT) } else { (regular(BODY_TEXT), DIM_TEXT) };
                    let text = ui.painter().layout_no_wrap(label.to_string(), font, ink);
                    let (pill, response) = ui.allocate_exact_size(Vec2::new(text.size().x + PILL_PADDING, TOPIC_HEIGHT), Sense::click());
                    points_to_when_hovered(ui, &response);
                    let fill = match (selected, response.hovered()) {
                        (true, _) => SELECTED_PILL,
                        (false, true) => Color32::from_white_alpha(HOVERED_PILL),
                        (false, false) => Color32::from_white_alpha(RESTING_PILL),
                    };
                    ui.painter().rect_filled(pill, TOPIC_HEIGHT / 2.0, fill);
                    ui.painter().galley(pill.center() - text.size() / 2.0, text, ink);
                    if response.clicked() {
                        picked = Some(index);
                    }
                }
                ui.add_space(MARGIN);
            });
        },
    );
    if let Some(topic) = picked {
        search.topic = topic;
        search.edited = None;
        ui.ctx().request_repaint();
    }
    ui.add_space(TOPICS_TO_ROWS);
}
