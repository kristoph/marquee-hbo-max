use eframe::egui::{self, Color32, CornerRadius, Image, Pos2, Rect, Vec2};
use max_api::cms::Badge;

use super::tile::TileScene;
use crate::{
    metrics::rail::{BADGE_HEIGHT, BADGE_ICON_HEIGHT, BANNER_HEIGHT, PROGRESS_BAR_HEIGHT, TILE_CORNER_RADIUS},
    paint::{play_mark, progress_bar},
    theme::{self, bold, LIVE_PROGRESS, TEXT},
};

const BADGE_TEXT: f32 = 12.5;
const BADGE_INSET: f32 = 7.0;
const BADGE_ICON_GAP: f32 = 4.0;
const BADGE_OUTER_CORNER: u8 = 2;
const BANNER_TEXT: f32 = 11.5;
const BANNER_PICTURE_MARGIN: f32 = 8.0;
const BANNER_PADDING: f32 = 18.0;
const BANNER_CORNER: u8 = 3;
const PROGRESS_TRACK: Color32 = Color32::from_gray(70);
const PLAY_MARK_FROM_LEFT: f32 = 24.0;
const PLAY_MARK_ABOVE_BAR: f32 = 22.0;
const PLAY_MARK_RADIUS: f32 = 10.0;

pub(super) fn paint(ui: &mut egui::Ui, scene: &TileScene) {
    if let Some(badge) = &scene.tile.detail.badge {
        paint_badge(ui, scene, badge);
    }
    if let Some(banner) = &scene.tile.detail.banner {
        paint_banner(ui, scene, banner);
    }
    if let Some(progress) = scene.tile.detail.progress {
        paint_progress(ui, scene, progress);
    }
}

fn paint_badge(ui: &mut egui::Ui, scene: &TileScene, badge: &Badge) {
    let ink = theme::from_rgba(badge.ink);
    let label = ui.painter().layout_no_wrap(badge.label.clone(), bold(BADGE_TEXT), ink);
    let icon = scene.tile.badge_icon.as_ref().and_then(|icon| Some((icon.uri()?, BADGE_ICON_HEIGHT * icon.aspect)));
    let icon_room = icon.map_or(0.0, |(_, width)| width + BADGE_ICON_GAP);
    let chip = Rect::from_min_size(scene.artwork.min, Vec2::new(label.size().x + icon_room + BADGE_INSET * 2.0, BADGE_HEIGHT));
    let corners = CornerRadius { nw: TILE_CORNER_RADIUS as u8, ne: 0, sw: 0, se: BADGE_OUTER_CORNER };
    ui.painter().rect_filled(chip, corners, theme::from_rgba(badge.background));
    if let Some((uri, width)) = icon {
        let corner = Pos2::new(chip.left() + BADGE_INSET, chip.center().y - BADGE_ICON_HEIGHT / 2.0);
        Image::new(uri).paint_at(ui, Rect::from_min_size(corner, Vec2::new(width, BADGE_ICON_HEIGHT)));
    }
    ui.painter().galley(Pos2::new(chip.left() + BADGE_INSET + icon_room, chip.center().y - label.size().y / 2.0), label, ink);
}

fn paint_banner(ui: &mut egui::Ui, scene: &TileScene, banner: &Badge) {
    let ink = theme::from_rgba(banner.ink);
    let picture_height = BANNER_HEIGHT - BANNER_PICTURE_MARGIN;
    let picture = scene.tile.banner_icon.as_ref().and_then(|icon| Some((icon.uri()?, picture_height * icon.aspect)));
    let words = ui.painter().layout_no_wrap(banner.label.clone(), bold(BANNER_TEXT), ink);
    let content_width = picture.map_or(words.size().x, |(_, width)| width);
    let center = Pos2::new(scene.artwork.center().x, scene.artwork.bottom() - BANNER_HEIGHT / 2.0);
    let chip = Rect::from_center_size(center, Vec2::new(content_width + BANNER_PADDING, BANNER_HEIGHT));
    let corners = CornerRadius { nw: BANNER_CORNER, ne: BANNER_CORNER, sw: 0, se: 0 };
    ui.painter().rect_filled(chip, corners, theme::from_rgba(banner.background));
    match picture {
        Some((uri, width)) => {
            Image::new(uri).paint_at(ui, Rect::from_center_size(chip.center(), Vec2::new(width, picture_height)));
        }
        None => {
            ui.painter().galley(chip.center() - words.size() / 2.0, words, ink);
        }
    }
}

fn paint_progress(ui: &mut egui::Ui, scene: &TileScene, progress: f32) {
    let artwork = scene.artwork;
    let track = Rect::from_min_max(Pos2::new(artwork.left(), artwork.bottom() - PROGRESS_BAR_HEIGHT), artwork.max);
    let fill = if scene.tile.detail.live { LIVE_PROGRESS } else { TEXT };
    progress_bar(ui.painter(), track, progress, 0.0, PROGRESS_TRACK, fill);
    let mark = Pos2::new(artwork.left() + PLAY_MARK_FROM_LEFT, track.top() - PLAY_MARK_ABOVE_BAR);
    play_mark(ui.painter(), mark, PLAY_MARK_RADIUS, TEXT);
}
