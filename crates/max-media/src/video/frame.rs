use objc2_core_video::{
    CVPixelBuffer, CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight, CVPixelBufferGetWidth,
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};

const BYTES_PER_PIXEL: usize = 4;

pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Frame {
    pub(super) fn copy_from_bgra(buffer: &CVPixelBuffer) -> Option<Self> {
        // SAFETY: the buffer is locked while read, and each row read is `width` pixels of the
        // `bytes_per_row` the buffer holds for it.
        unsafe {
            CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags::ReadOnly);
            let (width, height) = (CVPixelBufferGetWidth(buffer), CVPixelBufferGetHeight(buffer));
            let bytes_per_row = CVPixelBufferGetBytesPerRow(buffer);
            let base = CVPixelBufferGetBaseAddress(buffer) as *const u8;
            let mut rgba = Vec::with_capacity(width * height * BYTES_PER_PIXEL);
            if !base.is_null() {
                for row in 0..height {
                    let line = std::slice::from_raw_parts(base.add(row * bytes_per_row), width * BYTES_PER_PIXEL);
                    rgba.extend(line.as_chunks::<BYTES_PER_PIXEL>().0.iter().flat_map(|[blue, green, red, _]| [*red, *green, *blue, u8::MAX]));
                }
            }
            CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags::ReadOnly);
            (rgba.len() == width * height * BYTES_PER_PIXEL).then_some(Self { width, height, rgba })
        }
    }
}
