use std::time::{Duration, Instant};

use eframe::egui;

use crate::{app::App, model::Selection};

const STEP_EVERY: Duration = Duration::from_millis(140);
const REPORT_EVERY: Duration = Duration::from_secs(3);
const SLOW_FRAME_MILLISECONDS: f32 = 25.0;

pub struct Benchmark {
    last_step: Instant,
    last_report: Instant,
    last_frame: Option<Instant>,
    frame_processor_milliseconds: Vec<f32>,
    milliseconds_between_frames: Vec<f32>,
}

impl Benchmark {
    pub fn new() -> Self {
        Self {
            last_step: Instant::now(),
            last_report: Instant::now(),
            last_frame: None,
            frame_processor_milliseconds: Vec::new(),
            milliseconds_between_frames: Vec::new(),
        }
    }

    fn report(&mut self, tiles_drawn_before_their_artwork: u32) {
        let mut frames = std::mem::take(&mut self.frame_processor_milliseconds);
        frames.sort_by(f32::total_cmp);
        let mean = frames.iter().sum::<f32>() / frames.len() as f32;
        let ninety_fifth = frames[(frames.len() * 95 / 100).min(frames.len() - 1)];
        let gaps = std::mem::take(&mut self.milliseconds_between_frames);
        let slow = gaps.iter().filter(|gap| **gap > SLOW_FRAME_MILLISECONDS).count();
        let worst = gaps.iter().copied().fold(0.0, f32::max);
        log::info!(
            "PERF frames={} cpu mean={mean:.1}ms p95={ninety_fifth:.1}ms max={:.1}ms | frame gaps over {SLOW_FRAME_MILLISECONDS}ms: {slow}, worst {worst:.0}ms | tiles drawn before their artwork: {tiles_drawn_before_their_artwork}",
            frames.len(),
            frames[frames.len() - 1],
        );
        self.last_report = Instant::now();
    }
}

impl App {
    pub(crate) fn run_benchmark(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if self.developer.benchmark.is_none() || self.page.rows.is_empty() {
            return;
        }
        let next_row = self.page.row_after(self.page.selected.row).or_else(|| self.page.row_after(0)).unwrap_or(0);
        let Some(benchmark) = &mut self.developer.benchmark else { return };
        ctx.request_repaint();
        benchmark.frame_processor_milliseconds.extend(frame.info().cpu_usage.map(|seconds| seconds * 1000.0));
        let now = Instant::now();
        let since_last_frame = benchmark.last_frame.replace(now).map(|last| now.duration_since(last).as_secs_f32() * 1000.0);
        benchmark.milliseconds_between_frames.extend(since_last_frame);
        if benchmark.last_report.elapsed() >= REPORT_EVERY && !benchmark.frame_processor_milliseconds.is_empty() {
            benchmark.report(self.developer.tiles_drawn_before_their_artwork.replace(0));
        }
        if benchmark.last_step.elapsed() >= STEP_EVERY {
            benchmark.last_step = Instant::now();
            self.page.selected = Selection::new(next_row, self.page.selected.column.min(self.page.last_column(next_row)));
            self.page.reveal_selection();
        }
    }
}
