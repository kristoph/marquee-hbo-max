use eframe::egui::{self, Align2, Color32, Id, Pos2, Rect, Sense, Vec2};

use super::{TitleControl, SIDE_MARGIN};
use crate::{
    paint::{points_to_when_hovered, progress_bar},
    player::NativePlayer,
    theme::{bold, regular, TEXT},
};

const TITLE_ABOVE_FOOT: f32 = 167.0;
const TITLE_TEXT: f32 = 22.0;
const SUBTITLE_ABOVE_FOOT: f32 = 140.0;
const SUBTITLE_TEXT: f32 = 18.0;
const TIME_ABOVE_FOOT: f32 = 126.0;
const TIME_TEXT: f32 = 16.0;
const SCRUBBER_ABOVE_FOOT: f32 = 95.0;
const SCRUBBER_HEIGHT: f32 = 3.0;
const SCRUBBER_HEIGHT_ENGAGED: f32 = 5.0;
const SCRUBBER_TARGET_HEIGHT: f32 = 26.0;
const KNOB_RADIUS: f32 = 8.0;
const SCRUBBER_TRACK: u8 = 90;

pub(super) fn draw(ui: &mut egui::Ui, window: Rect, player: &NativePlayer) -> Option<TitleControl> {
    let (left, right, foot) = (window.left() + SIDE_MARGIN, window.right() - SIDE_MARGIN, window.bottom());
    let heading = player.heading();
    let painter = ui.painter();
    painter.text(Pos2::new(left, foot - TITLE_ABOVE_FOOT), Align2::LEFT_TOP, &heading.title, bold(TITLE_TEXT), TEXT);
    if let Some(subtitle) = &heading.subtitle {
        painter.text(Pos2::new(left, foot - SUBTITLE_ABOVE_FOOT), Align2::LEFT_TOP, subtitle, regular(SUBTITLE_TEXT), TEXT);
    }
    let (position, duration) = (player.position()?, player.duration()?);

    let middle = foot - SCRUBBER_ABOVE_FOOT;
    let target = Rect::from_min_max(Pos2::new(left, middle - SCRUBBER_TARGET_HEIGHT / 2.0), Pos2::new(right, middle + SCRUBBER_TARGET_HEIGHT / 2.0));
    let response = ui.interact(target, Id::new("title-scrubber"), Sense::click_and_drag());
    points_to_when_hovered(ui, &response);
    let engaged = response.hovered() || response.dragged();
    let pointed = response.interact_pointer_pos().map(|pointer| f64::from(((pointer.x - left) / target.width()).clamp(0.0, 1.0)));
    let shown = pointed.filter(|_| response.dragged()).unwrap_or(position / duration);

    let height = if engaged { SCRUBBER_HEIGHT_ENGAGED } else { SCRUBBER_HEIGHT };
    let track = Rect::from_center_size(target.center(), Vec2::new(target.width(), height));
    progress_bar(ui.painter(), track, shown as f32, height / 2.0, Color32::from_white_alpha(SCRUBBER_TRACK), TEXT);
    ui.painter().circle_filled(Pos2::new(track.left() + track.width() * shown as f32, middle), KNOB_RADIUS, TEXT);
    let remaining = format!("-{}", clock(duration - shown * duration));
    ui.painter().text(Pos2::new(right, foot - TIME_ABOVE_FOOT), Align2::RIGHT_CENTER, remaining, regular(TIME_TEXT), TEXT);

    let let_go = response.clicked() || response.drag_stopped();
    pointed.filter(|_| let_go).map(|fraction| TitleControl::SeekTo(fraction * duration))
}

fn clock(seconds: f64) -> String {
    let whole = seconds.max(0.0) as u64;
    let (hours, minutes, seconds) = (whole / 3600, whole / 60 % 60, whole % 60);
    match hours {
        0 => format!("{minutes}:{seconds:02}"),
        _ => format!("{hours}:{minutes:02}:{seconds:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::clock;

    #[test]
    fn writes_times_as_a_clock() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(84.9), "1:24");
        assert_eq!(clock(7760.8), "2:09:20");
    }
}
