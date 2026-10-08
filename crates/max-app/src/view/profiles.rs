use eframe::egui::{self, Align2, Color32, Id, Image, Key, Pos2, Rect, Sense, Stroke, StrokeKind, TextEdit, Vec2};

use crate::{
    app::{PinEntry, ProfilePicker},
    intent::{Intent, ProfileRequest},
    metrics::{
        profiles::{AVATAR, AVATAR_GAP, BUTTON_HEIGHT, NAME_TEXT, TITLE_TEXT},
        BODY_TEXT, MARGIN,
    },
    model::ScreenAccount,
    paint::{cross, points_to_when_hovered},
    theme::{bold, regular, DIM_TEXT, PLACEHOLDER, TEXT, TRANSLUCENT},
};

const TITLE: &str = "Who's Watching?";
const SIGN_OUT: &str = "Sign Out";
const PIN_LENGTH: usize = 4;
const TITLE_ABOVE_AVATARS: f32 = 70.0;
const NAME_BELOW_AVATAR: f32 = 18.0;
const SIGN_OUT_BELOW_NAMES: f32 = 90.0;
const SIGN_OUT_PADDING: f32 = 64.0;
const SELECTED_RING: f32 = 4.0;
const HOVERED_RING: f32 = 2.0;
const CLOSE_TARGET: f32 = 40.0;
const CLOSE_ARM: f32 = 9.0;
const PIN_FIELD: Vec2 = Vec2::new(180.0, 56.0);
const PIN_TEXT: f32 = 28.0;
const HOVERED_BUTTON: u8 = 80;
const DIMMED_WHILE_SWITCHING: f32 = 0.4;

pub(crate) fn draw(ui: &mut egui::Ui, picker: &mut ProfilePicker, intent: &mut Intent) {
    let window = ui.max_rect();
    let Some(account) = &picker.account else { return };
    let count = account.account.profiles.len() as f32;
    let row_width = count * AVATAR + (count - 1.0).max(0.0) * AVATAR_GAP;
    let avatars_top = window.center().y - AVATAR / 2.0;
    let painter = ui.painter().clone();
    painter.text(Pos2::new(window.center().x, avatars_top - TITLE_ABOVE_AVATARS), Align2::CENTER_CENTER, TITLE, bold(TITLE_TEXT), TEXT);
    if draw_close_button(ui, window) {
        intent.profile = Some(ProfileRequest::ClosePicker);
    }
    if let Some(entry) = &mut picker.pin_entry {
        return draw_pin_entry(ui, window, account, entry, intent);
    }

    let opacity = if picker.switching { DIMMED_WHILE_SWITCHING } else { 1.0 };
    for (index, profile) in account.account.profiles.iter().enumerate() {
        let left = window.center().x - row_width / 2.0 + index as f32 * (AVATAR + AVATAR_GAP);
        let avatar = Rect::from_min_size(Pos2::new(left, avatars_top), Vec2::splat(AVATAR));
        let response = ui.interact(avatar, Id::new(("profile", index)), Sense::click());
        points_to_when_hovered(ui, &response);
        match account.avatars.get(index).and_then(Option::as_ref) {
            Some(uri) => {
                Image::new(uri.as_str()).corner_radius(AVATAR / 2.0).tint(Color32::WHITE.gamma_multiply(opacity)).paint_at(ui, avatar);
            }
            None => {
                painter.circle_filled(avatar.center(), AVATAR / 2.0, PLACEHOLDER);
            }
        }
        let ring = if profile.selected {
            SELECTED_RING
        } else if response.hovered() {
            HOVERED_RING
        } else {
            0.0
        };
        if ring > 0.0 {
            painter.circle_stroke(avatar.center(), AVATAR / 2.0 + ring, Stroke::new(ring, TEXT));
        }
        let name = if profile.needs_pin { format!("{}  (PIN)", profile.name) } else { profile.name.clone() };
        let font = if profile.selected { bold(NAME_TEXT) } else { regular(NAME_TEXT) };
        painter.text(
            avatar.center_bottom() + Vec2::new(0.0, NAME_BELOW_AVATAR),
            Align2::CENTER_TOP,
            name,
            font,
            if profile.selected { TEXT } else { DIM_TEXT },
        );
        if response.clicked() && !picker.switching {
            intent.profile = Some(ProfileRequest::Choose(index));
        }
    }
    let sign_out_center = Pos2::new(window.center().x, avatars_top + AVATAR + SIGN_OUT_BELOW_NAMES + BUTTON_HEIGHT / 2.0);
    if draw_pill_button(ui, sign_out_center, SIGN_OUT, "sign-out") && !picker.switching {
        intent.profile = Some(ProfileRequest::SignOut);
    }
}

fn draw_close_button(ui: &mut egui::Ui, window: Rect) -> bool {
    let center = Pos2::new(window.right() - MARGIN - CLOSE_TARGET / 2.0, window.top() + MARGIN + CLOSE_TARGET / 2.0);
    let response = ui.interact(Rect::from_center_size(center, Vec2::splat(CLOSE_TARGET)), Id::new("profiles-close"), Sense::click());
    points_to_when_hovered(ui, &response);
    cross(ui.painter(), center, CLOSE_ARM, Stroke::new(2.0, if response.hovered() { TEXT } else { DIM_TEXT }));
    response.clicked()
}

fn draw_pill_button(ui: &mut egui::Ui, center: Pos2, label: &str, name: &str) -> bool {
    let text = ui.painter().layout_no_wrap(label.to_string(), bold(BODY_TEXT), TEXT);
    let button = Rect::from_center_size(center, Vec2::new(text.size().x + SIGN_OUT_PADDING, BUTTON_HEIGHT));
    let response = ui.interact(button, Id::new(("profiles-button", name)), Sense::click());
    points_to_when_hovered(ui, &response);
    let fill = if response.hovered() { Color32::from_white_alpha(HOVERED_BUTTON) } else { TRANSLUCENT };
    ui.painter().rect_filled(button, BUTTON_HEIGHT / 2.0, fill);
    ui.painter().rect_stroke(button, BUTTON_HEIGHT / 2.0, Stroke::new(1.0, DIM_TEXT), StrokeKind::Inside);
    ui.painter().galley(button.center() - text.size() / 2.0, text, TEXT);
    response.clicked()
}

fn draw_pin_entry(ui: &mut egui::Ui, window: Rect, account: &ScreenAccount, entry: &mut PinEntry, intent: &mut Intent) {
    let name = account.account.profiles.get(entry.profile).map(|profile| profile.name.as_str()).unwrap_or_default();
    let prompt = format!("Enter the PIN for {name}");
    ui.painter().text(window.center() - Vec2::new(0.0, PIN_FIELD.y), Align2::CENTER_CENTER, prompt, regular(NAME_TEXT), TEXT);
    let field = TextEdit::singleline(&mut entry.digits)
        .id(Id::new("profile-pin"))
        .password(true)
        .char_limit(PIN_LENGTH)
        .font(regular(PIN_TEXT))
        .horizontal_align(egui::Align::Center)
        .vertical_align(egui::Align::Center);
    let response = ui.put(Rect::from_center_size(window.center() + Vec2::new(0.0, PIN_FIELD.y / 2.0), PIN_FIELD), field);
    if std::mem::take(&mut entry.focus_field) {
        response.request_focus();
    }
    entry.digits.retain(|character| character.is_ascii_digit());
    let (entered, cancelled) = ui.input(|input| (input.key_pressed(Key::Enter), input.key_pressed(Key::Escape)));
    if cancelled {
        intent.profile = Some(ProfileRequest::CancelPin);
    } else if entered && entry.digits.len() == PIN_LENGTH {
        intent.profile = Some(ProfileRequest::SubmitPin);
    }
}
