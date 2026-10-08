use eframe::egui::{text::LayoutJob, FontId, Painter, Pos2, TextFormat, Vec2};

use super::tile::TileScene;
use crate::{
    metrics::{BODY_TEXT, SMALL_TEXT},
    paint::{ellipsis_after, WrappedText},
    theme::{bold, regular, DIM_TEXT, TEXT},
};

const BELOW_GRID_ARTWORK: f32 = 7.0;
const BELOW_RESUMED_ARTWORK: f32 = 9.0;
const BELOW_ARTWORK: f32 = 12.0;
const FACTS_SEPARATOR: &str = "   ";

pub(super) fn paint(painter: &Painter, scene: &TileScene) {
    if scene.row.grid_line {
        paint_title_and_facts(painter, scene);
    } else if scene.row.resumes_watching {
        paint_episode_and_rating(painter, scene);
    } else {
        paint_title_and_subtitle(painter, scene);
    }
}

fn one_line<'text>(text: &'text str, font: FontId, color: eframe::egui::Color32, scene: &TileScene) -> WrappedText<'text> {
    WrappedText { text, font, color, width: scene.base.width(), lines: 1 }
}

fn paint_title_and_facts(painter: &Painter, scene: &TileScene) {
    let top = Pos2::new(scene.base.left(), scene.artwork.bottom() + BELOW_GRID_ARTWORK);
    let title_height = one_line(&scene.tile.title, bold(SMALL_TEXT), DIM_TEXT, scene).paint(painter, top);
    let facts = scene.tile.detail.facts.join(FACTS_SEPARATOR);
    one_line(&facts, regular(SMALL_TEXT), DIM_TEXT, scene).paint(painter, top + Vec2::new(0.0, title_height + 1.0));
}

fn paint_episode_and_rating(painter: &Painter, scene: &TileScene) {
    let top = Pos2::new(scene.base.left(), scene.artwork.bottom() + BELOW_RESUMED_ARTWORK);
    let format = |font: FontId| TextFormat { font_id: font, color: DIM_TEXT, ..Default::default() };
    let mut line = LayoutJob { wrap: ellipsis_after(1, scene.base.width()), ..Default::default() };
    if let Some((season, episode)) = scene.tile.detail.season_and_episode {
        line.append(&format!("S{season} E{episode} "), 0.0, format(regular(SMALL_TEXT)));
    }
    line.append(&scene.tile.title, 0.0, format(bold(SMALL_TEXT)));
    let line = painter.layout_job(line);
    let line_height = line.size().y;
    painter.galley(top, line, DIM_TEXT);
    if let Some(rating) = &scene.tile.detail.rating {
        one_line(rating, regular(SMALL_TEXT), DIM_TEXT, scene).paint(painter, top + Vec2::new(0.0, line_height + 2.0));
    }
}

fn paint_title_and_subtitle(painter: &Painter, scene: &TileScene) {
    let top = Pos2::new(scene.base.left(), scene.artwork.bottom() + BELOW_ARTWORK);
    let title_height = one_line(&scene.tile.title, regular(BODY_TEXT), TEXT, scene).paint(painter, top);
    if let Some(subtitle) = scene.tile.detail.secondary_title.as_ref().filter(|subtitle| **subtitle != scene.tile.title) {
        one_line(subtitle, regular(SMALL_TEXT), DIM_TEXT, scene).paint(painter, top + Vec2::new(0.0, title_height + 3.0));
    }
}
