use std::{cell::Cell, time::Instant};

use objc2::{rc::Retained, runtime::AnyObject, AnyThread};
use objc2_av_foundation::{AVPlayer, AVPlayerItem, AVPlayerItemStatus, AVPlayerItemVideoOutput};
use objc2_core_video::{kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_32BGRA};
use objc2_foundation::{MainThreadMarker, NSDictionary, NSNumber, NSString, NSURL};

use super::{frame::Frame, time::seconds};

const END_TOLERANCE_SECONDS: f64 = 0.1;

pub struct Clip {
    pub(super) player: Retained<AVPlayer>,
    pub(super) item: Retained<AVPlayerItem>,
    output: Retained<AVPlayerItemVideoOutput>,
    pub(super) sound: Option<Retained<AVPlayer>>,
    pub(super) muted: Cell<bool>,
    pub(super) playing: Cell<bool>,
    pub(super) sound_started: Cell<bool>,
    pub(super) sound_checked: Cell<Instant>,
}

impl Clip {
    pub fn open(video_url: &str, sound_url: Option<&str>) -> Option<Self> {
        let main_thread = MainThreadMarker::new()?;
        // SAFETY: on the main thread; the attributes pair the pixel-format key, an NSString,
        // with an NSNumber, as the framework documents.
        unsafe {
            let video_url = NSURL::URLWithString(&NSString::from_str(video_url))?;
            let item = AVPlayerItem::playerItemWithURL(&video_url, main_thread);
            let format_key: &NSString = &*std::ptr::from_ref(kCVPixelBufferPixelFormatTypeKey).cast::<NSString>();
            let format = NSNumber::new_u32(kCVPixelFormatType_32BGRA);
            let attributes: Retained<NSDictionary<NSString, AnyObject>> =
                NSDictionary::from_slices(&[format_key], &[&*Retained::as_ptr(&format).cast::<AnyObject>()]);
            let output = AVPlayerItemVideoOutput::initWithPixelBufferAttributes(AVPlayerItemVideoOutput::alloc(), Some(&attributes));
            item.addOutput(&output);
            let player = AVPlayer::playerWithPlayerItem(Some(&item), main_thread);
            player.setMuted(true);
            player.play();
            let sound = sound_url.and_then(|url| NSURL::URLWithString(&NSString::from_str(url))).map(|url| {
                let sound = AVPlayer::playerWithURL(&url, main_thread);
                // Required before a player may be started at a given clock time.
                sound.setAutomaticallyWaitsToMinimizeStalling(false);
                sound
            });
            Some(Self {
                player,
                item,
                output,
                sound,
                muted: Cell::new(true),
                playing: Cell::new(true),
                sound_started: Cell::new(false),
                sound_checked: Cell::new(Instant::now()),
            })
        }
    }

    pub fn new_frame(&self) -> Option<Frame> {
        self.keep_sound_in_step();
        // SAFETY: on the main thread.
        unsafe {
            let time = self.item.currentTime();
            if !self.output.hasNewPixelBufferForItemTime(time) {
                return None;
            }
            let buffer = self.output.copyPixelBufferForItemTime_itemTimeForDisplay(time, std::ptr::null_mut())?;
            Frame::copy_from_bgra(&buffer)
        }
    }

    pub fn progress(&self) -> Option<f32> {
        // SAFETY: on the main thread.
        let (position, length) = unsafe { (seconds(self.item.currentTime())?, seconds(self.item.duration())?) };
        (length > 0.0).then(|| (position / length).clamp(0.0, 1.0) as f32)
    }

    pub fn finished(&self) -> bool {
        // SAFETY: on the main thread.
        unsafe {
            if self.item.status() == AVPlayerItemStatus::Failed {
                return true;
            }
            let (position, length) = (seconds(self.item.currentTime()), seconds(self.item.duration()));
            matches!((position, length), (Some(position), Some(length)) if length > 0.0 && position >= length - END_TOLERANCE_SECONDS)
        }
    }

    pub fn set_playing(&self, playing: bool) {
        if self.playing.replace(playing) == playing {
            return;
        }
        // SAFETY: on the main thread.
        unsafe {
            match playing {
                true => self.player.play(),
                false => self.player.pause(),
            }
        }
        self.apply_sound_state();
    }
}

impl Drop for Clip {
    fn drop(&mut self) {
        self.set_playing(false);
    }
}
