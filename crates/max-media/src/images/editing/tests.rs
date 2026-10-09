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
