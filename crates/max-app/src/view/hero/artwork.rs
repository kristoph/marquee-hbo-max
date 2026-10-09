use eframe::egui::{self, Color32, Event, Id, Image, MouseWheelUnit, Pos2, Rect, Sense, Vec2};

use super::HeroScene;
use crate::{
    app::App,
    intent::Intent,
    paint::{gradient, CornerColors},
    theme::{self, BACKGROUND},
    timing::HERO_SLIDE,
};

const POINTS_PER_WHEEL_NOTCH: f32 = 40.0;
const SIDEWAYS_WHEN_THIS_MUCH_WIDER_THAN_TALL: f32 = 2.0;
const SIDE_SHADE_FRACTION_OF_ARTWORK: f32 = 0.58;
const FOOT_SHADE_STARTS_AT_FRACTION: f32 = 0.56;
const SIDE_SHADE_OVER_ARTWORK: u8 = 225;

impl App {
    pub(super) fn paint_hero_artwork(&self, ui: &mut egui::Ui, scene: &HeroScene) {
        ui.set_clip_rect(scene.visible.intersect(scene.bounds).intersect(scene.artwork));
        let mut position = scene.artwork;
        if let Some(slide) = self.hero.slide.as_ref().filter(|slide| slide.started.elapsed() < HERO_SLIDE) {
            let remaining = 1.0 - slide.started.elapsed().as_secs_f32() / HERO_SLIDE.as_secs_f32();
            let side = if slide.forward { 1.0 } else { -1.0 };
            let width = scene.artwork.width();
            position = scene.artwork.translate(Vec2::new(side * width * remaining.powi(3), 0.0));
            if let Some(uri) = scene.row.tiles.get(slide.from).and_then(|previous| previous.artwork.as_ref()?.uri()) {
                Image::new(uri).paint_at(ui, position.translate(Vec2::new(-side * width, 0.0)));
            }
            ui.ctx().request_repaint();
        }
        if let Some(uri) = scene.tile.artwork.as_ref().and_then(|artwork| artwork.uri()) {
            Image::new(uri).paint_at(ui, position);
        }
        if let Some((picture, opacity)) = self.preview_picture() {
            Image::new(picture).tint(theme::white(opacity)).paint_at(ui, scene.artwork);
        }
        ui.set_clip_rect(scene.visible);
    }

    /// Reads each scroll event as the trackpad reported it, because the toolkit's own total is
    /// smoothed and the page has already consumed it.
    pub(super) fn collect_hero_swipes(&self, ui: &mut egui::Ui, scene: &HeroScene, intent: &mut Intent) {
        if scene.row.tiles.len() < 2 {
            return;
        }
        let drag = ui.interact(scene.flow, Id::new(("hero-swipe", self.navigation.generation)), Sense::drag());
        intent.hero_drag = drag.drag_delta().x;
        intent.hero_drag_ended = drag.drag_stopped();
        if !ui.rect_contains_pointer(scene.flow) || self.browse_menu_open {
            return;
        }
        intent.hero_sideways_scrolls = ui.input(|input| {
            let scrolls = input.events.iter().filter_map(|event| match event {
                Event::MouseWheel { unit: MouseWheelUnit::Point, delta, .. } => Some(*delta),
                Event::MouseWheel { delta, .. } => Some(*delta * POINTS_PER_WHEEL_NOTCH),
                _ => None,
            });
            scrolls.filter(|delta| delta.x.abs() > delta.y.abs() * SIDEWAYS_WHEN_THIS_MUCH_WIDER_THAN_TALL).map(|delta| delta.x).collect()
        });
    }
}

pub(super) fn paint_shading(scene: &HeroScene) {
    let clear = Color32::TRANSPARENT;
    let artwork_narrower_than_window = scene.artwork.left() > scene.bounds.left() + 1.0;
    let shade = if artwork_narrower_than_window { BACKGROUND } else { Color32::from_black_alpha(SIDE_SHADE_OVER_ARTWORK) };
    // In a window narrower than the artwork, its left is out of sight and the shade starts at the window's edge.
    let shown = scene.artwork.intersect(scene.bounds);
    let side = Rect::from_min_size(shown.min, Vec2::new(shown.width() * SIDE_SHADE_FRACTION_OF_ARTWORK, scene.bounds.height()));
    gradient(&scene.painter, side, CornerColors::left_to_right(shade, clear));
    let foot_top = scene.bounds.top() + scene.bounds.height() * FOOT_SHADE_STARTS_AT_FRACTION;
    let foot = Rect::from_min_max(Pos2::new(scene.bounds.left(), foot_top), scene.bounds.right_bottom());
    gradient(&scene.painter, foot, CornerColors::top_to_bottom(clear, BACKGROUND));
}
