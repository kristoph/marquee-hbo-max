use std::{
    io,
    path::{Path, PathBuf},
};

use crate::Error;

use super::cache::write_whole;

const BARELY_VISIBLE_ALPHA: u8 = 16;

/// Fractions of the image's width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpaqueBounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl OpaqueBounds {
    pub const WHOLE: Self = Self { left: 0.0, top: 0.0, right: 1.0, bottom: 1.0 };
}

/// A small icon drawn straight from a large source shimmers, because the graphics card skips
/// most of the source's pixels; this resamples it properly, once, beside the original.
pub fn scaled_copy(source: &Path, width: u32, height: u32) -> Result<PathBuf, Error> {
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the image's path has no name"))?;
    let target = source.with_file_name(format!("{stem}_{width}x{height}.png"));
    if !target.exists() {
        let scaled = image::open(source)?.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        write_whole(&target, |partial| Ok(scaled.save_with_format(partial, image::ImageFormat::Png)?))?;
    }
    Ok(target)
}

pub fn opaque_bounds(path: &Path) -> Option<OpaqueBounds> {
    let image = image::open(path).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    let mut extent: Option<(u32, u32, u32, u32)> = None;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] > BARELY_VISIBLE_ALPHA {
            let (left, top, right, bottom) = extent.unwrap_or((x, y, x, y));
            extent = Some((left.min(x), top.min(y), right.max(x), bottom.max(y)));
        }
    }
    let (left, top, right, bottom) = extent?;
    Some(OpaqueBounds {
        left: left as f32 / width as f32,
        top: top as f32 / height as f32,
        right: (right + 1) as f32 / width as f32,
        bottom: (bottom + 1) as f32 / height as f32,
    })
}

#[cfg(test)]
mod tests;
