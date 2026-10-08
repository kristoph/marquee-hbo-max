use std::time::{Duration, Instant};

use eframe::egui::{self, Event, ViewportCommand};

use crate::app::App;

const WAIT_FOR_TEXTURES: Duration = Duration::from_secs(4);
const POLL: Duration = Duration::from_millis(200);

impl App {
    pub(crate) fn take_screenshot(&mut self, ctx: &egui::Context) {
        let Some(screenshot) = &mut self.developer.screenshot else { return };
        if self.page.rows.is_empty() {
            return;
        }
        let due = *screenshot.due.get_or_insert_with(|| Instant::now() + WAIT_FOR_TEXTURES);
        if Instant::now() < due {
            return ctx.request_repaint_after(POLL);
        }
        let taken = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        let Some(image) = taken else {
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default()));
            return ctx.request_repaint();
        };
        let [width, height] = image.size;
        match image::save_buffer(&screenshot.path, image.as_raw(), width as u32, height as u32, image::ColorType::Rgba8) {
            Ok(()) => log::info!("SCREENSHOT {}", screenshot.path),
            Err(error) => log::warn!("screenshot failed: {error}"),
        }
        self.developer.screenshot = None;
    }
}
