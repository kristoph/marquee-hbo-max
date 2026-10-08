use eframe::egui::{self, Align2, Color32, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use super::HeroScene;
use crate::{
    app::App,
    hero::HeroPreview,
    intent::{AccountChange, AccountChangeKind, Go, Intent},
    metrics::{hero::BUTTON_HEIGHT, CORNER_RADIUS, MARGIN},
    model::Selection,
    paint::{play_mark, plus, points_to_when_hovered, tick},
    theme::{self, bold, icon, TEXT, TRANSLUCENT},
};

const LABEL_TEXT: f32 = 20.0;
const LABEL_INSET: f32 = 26.0;
const PLAY_MARK_ROOM: f32 = 28.0;
const PLAY_MARK_INSET: f32 = 36.0;
const PLAY_MARK_RADIUS: f32 = 8.0;
const BUTTON_PADDING: f32 = 56.0;
const BUTTON_GAP: f32 = 14.0;
const FOCUS_RING_GAP: f32 = 4.0;
const FOCUS_RING_WIDTH: f32 = 2.5;
const LIST_MARK_SIZE: f32 = 9.0;
const SOUND_GLYPH: f32 = 22.0;
const SOUND_WORD: f32 = 14.0;
const HOVERED_PLAY_BUTTON: Color32 = Color32::from_gray(222);
const HOVERED_SQUARE_BUTTON: u8 = 80;

impl App {
    pub(super) fn draw_hero_buttons(&self, ui: &mut egui::Ui, scene: &HeroScene, intent: &mut Intent) {
        let after_play_button = self.draw_play_button(ui, scene, intent).unwrap_or(scene.left());
        self.draw_my_list_button(ui, scene, after_play_button, intent);
        self.draw_sound_button(ui, scene, intent);
    }

    fn draw_play_button(&self, ui: &mut egui::Ui, scene: &HeroScene, intent: &mut Intent) -> Option<f32> {
        let play_action = scene.tile.play_action();
        let action = play_action.or(scene.tile.detail.actions.first())?;
        let painter = &scene.painter;
        let label = painter.layout_no_wrap(action.label.clone(), bold(LABEL_TEXT), Color32::BLACK);
        let mark_room = if play_action.is_some() { PLAY_MARK_ROOM } else { 0.0 };
        let size = Vec2::new(label.size().x + mark_room + BUTTON_PADDING, BUTTON_HEIGHT);
        let button = Rect::from_min_size(Pos2::new(scene.left(), scene.button_top()), size);
        let response = ui.interact(button, Id::new(("hero-play", self.navigation.generation)), Sense::click());
        points_to_when_hovered(ui, &response);
        painter.rect_filled(button, CORNER_RADIUS, if response.hovered() { HOVERED_PLAY_BUTTON } else { Color32::WHITE });
        if play_action.is_some() {
            play_mark(painter, Pos2::new(button.left() + PLAY_MARK_INSET, button.center().y), PLAY_MARK_RADIUS, Color32::BLACK);
        }
        painter.galley(Pos2::new(button.left() + LABEL_INSET + mark_room, button.center().y - label.size().y / 2.0), label, Color32::BLACK);
        if scene.selected {
            let ring = Stroke::new(FOCUS_RING_WIDTH, TEXT);
            painter.rect_stroke(button.expand(FOCUS_RING_GAP), CORNER_RADIUS + FOCUS_RING_GAP, ring, StrokeKind::Outside);
        }
        if let Some(route) = action.route.as_ref().or(scene.tile.route.as_ref()).filter(|_| response.clicked()) {
            intent.go = Some(Go::to(route, &scene.tile.title));
        }
        Some(button.right() + BUTTON_GAP)
    }

    fn draw_my_list_button(&self, ui: &mut egui::Ui, scene: &HeroScene, left: f32, intent: &mut Intent) {
        let Some(listed) = scene.tile.is_on_my_list() else { return };
        let button = Rect::from_min_size(Pos2::new(left, scene.button_top()), Vec2::splat(BUTTON_HEIGHT));
        let response = self.square_hero_button(ui, scene, button, "hero-list");
        let stroke = Stroke::new(2.0, TEXT);
        match listed {
            true => tick(&scene.painter, button.center(), LIST_MARK_SIZE, stroke),
            false => plus(&scene.painter, button.center(), LIST_MARK_SIZE, stroke),
        }
        if response.clicked() {
            intent.account_change = Some(AccountChange { tile: Selection::new(0, scene.index), kind: AccountChangeKind::ToggleMyList });
        }
    }

    fn draw_sound_button(&self, ui: &mut egui::Ui, scene: &HeroScene, intent: &mut Intent) {
        if !self.hero.preview.as_ref().is_some_and(HeroPreview::is_on_screen) {
            return;
        }
        let corner = Pos2::new(scene.bounds.right() - MARGIN - BUTTON_HEIGHT, scene.button_top());
        let button = Rect::from_min_size(corner, Vec2::splat(BUTTON_HEIGHT));
        let response = self.square_hero_button(ui, scene, button, "hero-sound");
        let muted = self.hero.preview_muted;
        match theme::has_icons() {
            true => {
                let glyph = if muted { icon::SOUND_OFF } else { icon::SOUND_ON };
                scene.painter.text(button.center(), Align2::CENTER_CENTER, glyph, theme::icons(SOUND_GLYPH), TEXT)
            }
            false => scene.painter.text(button.center(), Align2::CENTER_CENTER, if muted { "Off" } else { "On" }, bold(SOUND_WORD), TEXT),
        };
        if response.clicked() {
            intent.toggle_preview_sound = true;
        }
    }

    fn square_hero_button(&self, ui: &mut egui::Ui, scene: &HeroScene, button: Rect, name: &str) -> egui::Response {
        let response = ui.interact(button, Id::new((name, self.navigation.generation)), Sense::click());
        points_to_when_hovered(ui, &response);
        let fill = if response.hovered() { Color32::from_white_alpha(HOVERED_SQUARE_BUTTON) } else { TRANSLUCENT };
        scene.painter.rect_filled(button, CORNER_RADIUS, fill);
        response
    }
}
