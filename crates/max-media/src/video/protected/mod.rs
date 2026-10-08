//! A protected title played by the system's player: the picture goes straight to a layer in
//! the window, and the key that unlocks it never leaves the system's keeping.

mod keys;
mod server;

use std::{collections::HashMap, ffi::c_void, sync::Arc};

use crate::Error;
pub use keys::FetchKey;
use keys::KeyDelegate;
use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
};
use objc2_av_foundation::{
    AVContentKeyRecipient, AVContentKeySession, AVContentKeySystemFairPlayStreaming, AVLayerVideoGravityResizeAspect, AVPlayer, AVPlayerItem,
    AVPlayerItemStatus, AVPlayerLayer, AVURLAsset,
};
use objc2_core_graphics::CGColor;
use objc2_core_media::CMTime;
use objc2_foundation::{MainThreadMarker, NSString, NSURL};
use objc2_quartz_core::{CAAutoresizingMask, CALayer};
use server::PlaylistServer;

use super::time::seconds;

const KEY_QUEUE: &str = "max.fairplay-keys";
const END_TOLERANCE_SECONDS: f64 = 0.5;
const SEEK_TIMESCALE: i32 = 600;

pub struct ProtectedTitle {
    pub playlists: HashMap<String, String>,
    pub master_playlist: String,
    pub certificate: Vec<u8>,
    pub fetch_key: Arc<FetchKey>,
}

pub struct ProtectedVideo {
    player: Retained<AVPlayer>,
    item: Retained<AVPlayerItem>,
    layer: Retained<AVPlayerLayer>,
    _key_session: Retained<AVContentKeySession>,
    _key_delegate: Retained<KeyDelegate>,
    _server: PlaylistServer,
}

impl ProtectedVideo {
    pub fn open(title: ProtectedTitle) -> Result<Self, Error> {
        let main_thread = MainThreadMarker::new().ok_or(Error::Video("it was not asked for on the main thread"))?;
        let server = PlaylistServer::serve(title.playlists)?;
        let url = NSURL::URLWithString(&NSString::from_str(&server.url(&title.master_playlist)))
            .ok_or(Error::Video("its playlist address is not a URL"))?;
        let key_delegate = KeyDelegate::new(title.certificate, title.fetch_key);
        let key_queue = dispatch2::DispatchQueue::new(KEY_QUEUE, dispatch2::DispatchQueueAttr::SERIAL);
        // SAFETY: on the main thread; the delegate and the queue outlive the session, which
        // this value keeps alongside them.
        unsafe {
            let asset = AVURLAsset::URLAssetWithURL_options(&url, None);
            let key_session = AVContentKeySession::contentKeySessionWithKeySystem(AVContentKeySystemFairPlayStreaming);
            key_session.setDelegate_queue(Some(ProtocolObject::from_ref(&*key_delegate)), Some(&key_queue));
            key_session.addContentKeyRecipient(ProtocolObject::<dyn AVContentKeyRecipient>::from_ref(&*asset));
            let item = AVPlayerItem::playerItemWithAsset(&asset, main_thread);
            let player = AVPlayer::playerWithPlayerItem(Some(&item), main_thread);
            let layer = AVPlayerLayer::playerLayerWithPlayer(Some(&player));
            if let Some(whole_picture_inside_the_window) = AVLayerVideoGravityResizeAspect {
                layer.setVideoGravity(whole_picture_inside_the_window);
            }
            Ok(Self { player, item, layer, _key_session: key_session, _key_delegate: key_delegate, _server: server })
        }
    }

    /// Puts the picture behind what the window's view draws, so that the app can draw its
    /// controls over the video wherever it leaves the view transparent.
    ///
    /// # Safety
    /// `view` must point to a live `NSView` that has a layer, and this must be the main thread.
    pub unsafe fn show_in(&self, view: *mut c_void) {
        let view = view.cast::<AnyObject>();
        let drawn_by_app: Option<Retained<CALayer>> = msg_send![view, layer];
        let Some(drawn_by_app) = drawn_by_app else { return };
        self.layer.setBackgroundColor(Some(&CGColor::new_generic_gray(0.0, 1.0)));
        self.layer.setAutoresizingMask(CAAutoresizingMask::LayerWidthSizable | CAAutoresizingMask::LayerHeightSizable);
        match drawn_by_app.superlayer() {
            Some(behind) => {
                self.layer.setFrame(drawn_by_app.frame());
                behind.insertSublayer_below(&self.layer, Some(&drawn_by_app));
            }
            None => {
                self.layer.setFrame(drawn_by_app.bounds());
                drawn_by_app.addSublayer(&self.layer);
            }
        }
    }

    pub fn set_playing(&self, playing: bool) {
        // SAFETY: on the main thread.
        unsafe {
            match playing {
                true => self.player.play(),
                false => self.player.pause(),
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        // SAFETY: on the main thread.
        unsafe { self.player.rate() > 0.0 }
    }

    pub fn set_muted(&self, muted: bool) {
        // SAFETY: on the main thread.
        unsafe { self.player.setMuted(muted) }
    }

    pub fn set_volume(&self, volume: f32) {
        // SAFETY: on the main thread.
        unsafe { self.player.setVolume(volume.clamp(0.0, 1.0)) }
    }

    pub fn position(&self) -> Option<f64> {
        // SAFETY: on the main thread.
        seconds(unsafe { self.item.currentTime() })
    }

    pub fn duration(&self) -> Option<f64> {
        // SAFETY: on the main thread.
        seconds(unsafe { self.item.duration() })
    }

    pub fn seek(&self, position_seconds: f64) {
        // SAFETY: on the main thread.
        unsafe { self.player.seekToTime(CMTime::with_seconds(position_seconds, SEEK_TIMESCALE)) }
    }

    pub fn has_picture(&self) -> bool {
        // SAFETY: on the main thread.
        unsafe { self.layer.isReadyForDisplay() }
    }

    pub fn failure(&self) -> Option<String> {
        // SAFETY: on the main thread.
        unsafe {
            (self.item.status() == AVPlayerItemStatus::Failed).then(|| {
                self.item.error().map_or_else(|| "the title could not be played".to_string(), |error| error.localizedDescription().to_string())
            })
        }
    }

    pub fn finished(&self) -> bool {
        matches!((self.position(), self.duration()), (Some(position), Some(duration)) if duration > 0.0 && position >= duration - END_TOLERANCE_SECONDS)
    }
}

impl Drop for ProtectedVideo {
    fn drop(&mut self) {
        self.set_playing(false);
        self.layer.removeFromSuperlayer();
    }
}
