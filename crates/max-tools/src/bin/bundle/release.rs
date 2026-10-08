use std::{
    fs, io,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{run, APP_NAME};

const APPLICATIONS: &str = "/Applications";

/// The hardened runtime and a timestamp are what Apple asks of an app before it will
/// notarise it.
pub fn sign(target: &Path, identity: &str) -> io::Result<()> {
    run(Command::new("codesign").args(["--force", "--options", "runtime", "--timestamp", "--sign", identity]).arg(target))?;
    run(Command::new("codesign").args(["--verify", "--strict"]).arg(target)).map(drop)
}

/// The image holds the app beside a link to the Applications folder, to be dragged onto it.
pub fn disk_image(app: &Path, identity: Option<&str>) -> io::Result<PathBuf> {
    let folder = app.parent().ok_or_else(|| io::Error::other("the app has no folder"))?;
    let (staging, image) = (folder.join("disk-image"), folder.join(format!("{APP_NAME}.dmg")));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;
    run(Command::new("ditto").arg(app).arg(staging.join(format!("{APP_NAME}.app"))))?;
    symlink(APPLICATIONS, staging.join("Applications"))?;
    run(Command::new("hdiutil").args(["create", "-ov", "-format", "UDZO", "-volname", APP_NAME, "-srcfolder"]).arg(&staging).arg(&image))?;
    fs::remove_dir_all(staging)?;
    if let Some(identity) = identity {
        sign(&image, identity)?;
    }
    Ok(image)
}

/// Apple's answer is attached to the image so that it opens without asking Apple again.
pub fn notarize(image: &Path, keychain_profile: &str) -> io::Result<()> {
    let answer = run(Command::new("xcrun").args(["notarytool", "submit", "--wait", "--keychain-profile", keychain_profile]).arg(image))?;
    println!("{}", answer.trim());
    run(Command::new("xcrun").args(["stapler", "staple"]).arg(image)).map(drop)
}
