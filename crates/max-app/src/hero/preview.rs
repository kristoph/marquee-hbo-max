use std::{thread, time::Instant};

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use max_api::{client::PreviewFiles, Error};
use max_media::video::Clip;

use super::HeroTitle;
use crate::{
    app::App,
    message::Message,
    timing::{fraction_elapsed, PREVIEW_DELAY, PREVIEW_FADE},
};

pub struct HeroPreview {
    pub title: HeroTitle,
    pub stage: PreviewStage,
}

pub enum PreviewStage {
    Finding,
    Playing { clip: Clip, picture: Option<TextureHandle>, first_frame_at: Option<Instant> },
    Ended { picture: Option<TextureHandle>, since: Instant },
    Unavailable,
}

impl HeroPreview {
    pub fn is_on_screen(&self) -> bool {
        matches!(self.stage, PreviewStage::Playing { first_frame_at: Some(_), .. })
    }
}

impl App {
    pub(crate) fn run_preview(&mut self, ctx: &egui::Context) {
        let title = self.hero_title();
        if self.hero.shown_title != title {
            self.hero.shown_title = title;
            self.hero.shown_since = Instant::now();
        }
        if self.hero.preview.as_ref().is_some_and(|preview| preview.title != title) || self.playback.is_open() {
            self.hero.preview = None;
        }
        let watchable = self.hero_can_be_watched();
        match &mut self.hero.preview {
            None => self.find_preview(ctx, title, watchable),
            Some(HeroPreview { stage: PreviewStage::Playing { clip, picture, first_frame_at }, .. }) => {
                clip.set_playing(watchable);
                if let Some(frame) = clip.new_frame() {
                    let image = ColorImage::from_rgba_premultiplied([frame.width, frame.height], &frame.rgba);
                    match picture {
                        Some(texture) => texture.set(image, TextureOptions::LINEAR),
                        None => *picture = Some(ctx.load_texture("hero-preview", image, TextureOptions::LINEAR)),
                    }
                    if first_frame_at.is_none() {
                        *first_frame_at = Some(Instant::now());
                        clip.restart_sound_in_step();
                    }
                }
                if clip.finished() {
                    let stage = PreviewStage::Ended { picture: picture.take(), since: Instant::now() };
                    self.hero.preview = Some(HeroPreview { title, stage });
                }
                ctx.request_repaint();
            }
            Some(HeroPreview { stage: PreviewStage::Ended { picture, since }, .. }) => {
                if picture.is_some() && since.elapsed() < PREVIEW_FADE {
                    ctx.request_repaint();
                } else {
                    *picture = None;
                }
            }
            Some(HeroPreview { stage: PreviewStage::Finding | PreviewStage::Unavailable, .. }) => {}
        }
    }

    fn find_preview(&mut self, ctx: &egui::Context, title: HeroTitle, watchable: bool) {
        let Some(client) = self.service.client().filter(|_| watchable) else { return };
        let Some(edit_id) = self.hero_tile().and_then(|tile| tile.detail.preview_edit_id.clone()) else { return };
        let shown_for = self.hero.shown_since.elapsed();
        if shown_for < PREVIEW_DELAY {
            return ctx.request_repaint_after(PREVIEW_DELAY.saturating_sub(shown_for));
        }
        self.hero.preview = Some(HeroPreview { title, stage: PreviewStage::Finding });
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            let files = client.preview_files(&edit_id);
            let _ = sender.send(Message::Preview { title, files });
            ctx.request_repaint();
        });
    }

    pub(crate) fn start_preview(&mut self, title: HeroTitle, files: Result<PreviewFiles, Error>) {
        let muted = self.hero.preview_muted;
        let Some(preview) = self.hero.preview.as_mut().filter(|preview| preview.title == title) else { return };
        preview.stage = match files.map(|files| Clip::open(&files.video, files.audio.as_deref())) {
            Ok(Some(clip)) => {
                clip.set_muted(muted);
                PreviewStage::Playing { clip, picture: None, first_frame_at: None }
            }
            Ok(None) => PreviewStage::Unavailable,
            Err(reason) => {
                log::warn!("preview failed: {reason}");
                PreviewStage::Unavailable
            }
        };
    }

    pub(crate) fn toggle_preview_sound(&mut self) {
        self.hero.preview_muted = !self.hero.preview_muted;
        if let Some(HeroPreview { stage: PreviewStage::Playing { clip, .. }, .. }) = &self.hero.preview {
            clip.set_muted(self.hero.preview_muted);
        }
    }

    pub(crate) fn preview_picture(&self) -> Option<(&TextureHandle, f32)> {
        match &self.hero.preview.as_ref()?.stage {
            PreviewStage::Playing { picture: Some(picture), first_frame_at: Some(first_frame_at), .. } => {
                Some((picture, fraction_elapsed(*first_frame_at, PREVIEW_FADE)))
            }
            PreviewStage::Ended { picture: Some(picture), since } => Some((picture, 1.0 - fraction_elapsed(*since, PREVIEW_FADE))),
            _ => None,
        }
    }
}
