use eframe::egui::{self, Color32, Id, Pos2, Rect, Sense, Vec2};

use crate::{
    intent::Intent,
    metrics::{BODY_TEXT, CORNER_RADIUS},
    paint::{points_to_when_hovered, WrappedText},
    theme::{bold, regular, DIM_TEXT, PANEL, TEXT},
};

const HEADING: &str = "About";
const PARAGRAPHS: [&str; 3] = [
    "This app is an independent effort by Kristoph Cichocki-Romanov (hello@kristoph.net).",
    "It is not associated with Skydance Entertainment, Warner Bros. Discovery or HBO.",
    "Its text is set in Noto Sans and its icons come from Font Awesome, both used under the SIL Open Font License 1.1.",
];
const VERSION: &str = concat!("Version ", env!("CARGO_PKG_VERSION"));
const CLOSE: &str = "Close";

const SCRIM: u8 = 170;
const WIDTH: f32 = 560.0;
const PADDING: f32 = 40.0;
const HEADING_TEXT: f32 = 26.0;
const SPACE_BELOW_HEADING: f32 = 22.0;
const SPACE_BETWEEN_PARAGRAPHS: f32 = 14.0;
const LINES_PER_PARAGRAPH: usize = 4;
const SPACE_ABOVE_FOOT: f32 = 26.0;
const BUTTON: Vec2 = Vec2::new(120.0, 44.0);
const HOVERED_BUTTON: Color32 = Color32::from_gray(222);

/// Laid out once to find the panel's height, then painted; the panel closes on a click
/// anywhere, its button being only the obvious place to click.
pub(crate) fn draw(ui: &mut egui::Ui, window: Rect, intent: &mut Intent) {
    let scrim = ui.interact(window, Id::new("about-scrim"), Sense::click());
    let painter = ui.painter().clone();
    painter.rect_filled(window, 0.0, Color32::from_black_alpha(SCRIM));

    let paragraph =
        |text: &'static str| WrappedText { text, font: regular(BODY_TEXT), color: TEXT, width: WIDTH - PADDING * 2.0, lines: LINES_PER_PARAGRAPH };
    let text_height: f32 = PARAGRAPHS.iter().map(|text| paragraph(text).layout(&painter).size().y + SPACE_BETWEEN_PARAGRAPHS).sum();
    let height = PADDING + HEADING_TEXT + SPACE_BELOW_HEADING + text_height + SPACE_ABOVE_FOOT + BUTTON.y + PADDING;
    let panel = Rect::from_center_size(window.center(), Vec2::new(WIDTH, height));
    painter.rect_filled(panel, CORNER_RADIUS, PANEL);

    let mut pen = panel.min + Vec2::splat(PADDING);
    painter.text(pen, egui::Align2::LEFT_TOP, HEADING, bold(HEADING_TEXT), TEXT);
    pen.y += HEADING_TEXT + SPACE_BELOW_HEADING;
    for text in PARAGRAPHS {
        pen.y += paragraph(text).paint(&painter, pen) + SPACE_BETWEEN_PARAGRAPHS;
    }

    let foot = panel.bottom() - PADDING - BUTTON.y / 2.0;
    painter.text(Pos2::new(pen.x, foot), egui::Align2::LEFT_CENTER, VERSION, regular(BODY_TEXT), DIM_TEXT);
    let button = Rect::from_center_size(Pos2::new(panel.right() - PADDING - BUTTON.x / 2.0, foot), BUTTON);
    let close = ui.interact(button, Id::new("about-close"), Sense::click());
    points_to_when_hovered(ui, &close);
    painter.rect_filled(button, CORNER_RADIUS, if close.hovered() { HOVERED_BUTTON } else { Color32::WHITE });
    painter.text(button.center(), egui::Align2::CENTER_CENTER, CLOSE, bold(BODY_TEXT), Color32::BLACK);

    if scrim.clicked() || close.clicked() {
        intent.about = Some(false);
    }
}
