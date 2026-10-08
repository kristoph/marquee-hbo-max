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
mod tests {
    use std::fs;

    use super::*;

    fn scratch_folder(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("max-media-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    #[test]
    fn scaled_copy_has_the_requested_size() {
        let folder = scratch_folder("scaled");
        let source = folder.join("icon.png");
        image::RgbaImage::from_pixel(200, 100, image::Rgba([255, 0, 0, 255])).save(&source).unwrap();
        let copy = scaled_copy(&source, 56, 28).unwrap();
        assert_eq!(copy, folder.join("icon_56x28.png"));
        assert_eq!(image::image_dimensions(&copy).unwrap(), (56, 28));
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn finds_the_opaque_part_of_an_image() {
        let folder = scratch_folder("bounds");
        let path = folder.join("logo.png");
        let mut logo = image::RgbaImage::new(100, 40);
        for (x, y) in (10..30).flat_map(|x| (8..40).map(move |y| (x, y))) {
            logo.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
        logo.save(&path).unwrap();
        assert_eq!(opaque_bounds(&path), Some(OpaqueBounds { left: 0.1, top: 0.2, right: 0.3, bottom: 1.0 }));
        image::RgbaImage::new(4, 4).save(&path).unwrap();
        assert_eq!(opaque_bounds(&path), None);
        fs::remove_dir_all(&folder).unwrap();
    }
}
