use std::path::Path;

use eframe::egui::{self, Align2, Image, Pos2, Rect, Vec2};
use max_media::images::{opaque_bounds, OpaqueBounds};

use super::HeroScene;
use crate::{
    app::App,
    metrics::{
        hero::{LOGO_SPACE, TEXT_WIDTH},
        BODY_TEXT, SMALL_TEXT,
    },
    model::SizedImage,
    paint::WrappedText,
    theme::{bold, regular, DIM_TEXT, TEXT},
};

const TEXT_ABOVE_BUTTONS: f32 = 14.0;
const TITLE_ABOVE_TEXT: f32 = 14.0;
const LINE_GAP: f32 = 4.0;
const FACTS_LINE: f32 = 17.0;
const FACT_GAP: f32 = 10.0;
const HIGHLIGHT_LINE: f32 = 20.0;
const TITLE_TEXT: f32 = 52.0;
const TITLE_WIDTH_OVER_TEXT_WIDTH: f32 = 1.4;

impl App {
    pub(super) fn paint_hero_text(&self, ui: &mut egui::Ui, scene: &HeroScene) {
        let (painter, detail, left) = (&scene.painter, &scene.tile.detail, scene.left());
        let mut top = scene.button_top() - TEXT_ABOVE_BUTTONS;

        if let Some(description) = &detail.description {
            let text = WrappedText { text: description, font: regular(BODY_TEXT), color: TEXT, width: TEXT_WIDTH, lines: 3 };
            let galley = text.layout(painter);
            top -= galley.size().y;
            painter.galley(Pos2::new(left, top), galley, TEXT);
            top -= LINE_GAP;
        }
        let mut facts: Vec<&str> = detail.rating.iter().chain(&detail.genres).map(String::as_str).collect();
        if facts.is_empty() {
            facts.extend(detail.secondary_title.as_deref());
        }
        if !facts.is_empty() {
            top -= FACTS_LINE;
            let mut fact_left = left;
            for fact in facts {
                fact_left = painter.text(Pos2::new(fact_left, top), Align2::LEFT_TOP, fact, regular(SMALL_TEXT), DIM_TEXT).right() + FACT_GAP;
            }
            top -= LINE_GAP;
        }
        if let Some(highlight) = &detail.highlight {
            top -= HIGHLIGHT_LINE;
            painter.text(Pos2::new(left, top), Align2::LEFT_TOP, highlight, bold(BODY_TEXT), TEXT);
        }
        top -= TITLE_ABOVE_TEXT;

        match scene.tile.logo.as_ref().and_then(|logo| Some((logo, logo.uri()?))) {
            Some((logo, uri)) => self.paint_title_logo(ui, logo, uri, Pos2::new(left, top)),
            None => {
                let title = WrappedText {
                    text: &scene.tile.title,
                    font: bold(TITLE_TEXT),
                    color: TEXT,
                    width: TEXT_WIDTH * TITLE_WIDTH_OVER_TEXT_WIDTH,
                    lines: 2,
                };
                let galley = title.layout(painter);
                painter.galley(Pos2::new(left, top - galley.size().y), galley, TEXT);
            }
        }
    }

    fn paint_title_logo(&self, ui: &mut egui::Ui, logo: &SizedImage, uri: &str, bottom_left: Pos2) {
        let opaque = self.logo_bounds(uri);
        let shape = logo.aspect * (opaque.right - opaque.left) / (opaque.bottom - opaque.top);
        let size = match shape > LOGO_SPACE.x / LOGO_SPACE.y {
            true => Vec2::new(LOGO_SPACE.x, LOGO_SPACE.x / shape),
            false => Vec2::new(LOGO_SPACE.y * shape, LOGO_SPACE.y),
        };
        let part = Rect::from_min_max(Pos2::new(opaque.left, opaque.top), Pos2::new(opaque.right, opaque.bottom));
        Image::new(uri).uv(part).paint_at(ui, Rect::from_min_size(Pos2::new(bottom_left.x, bottom_left.y - size.y), size));
    }

    fn logo_bounds(&self, uri: &str) -> OpaqueBounds {
        let mut known = self.logo_bounds.borrow_mut();
        *known.entry(uri.to_string()).or_insert_with(|| opaque_bounds(Path::new(uri.trim_start_matches("file://"))).unwrap_or(OpaqueBounds::WHOLE))
    }
}
