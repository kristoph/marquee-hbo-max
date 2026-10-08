use objc2_core_media::CMTime;

pub(super) fn seconds(time: CMTime) -> Option<f64> {
    (time.timescale > 0).then(|| time.value as f64 / f64::from(time.timescale))
}
