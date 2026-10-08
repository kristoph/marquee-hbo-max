use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use max_api::workspace;

use crate::{run, APP_NAME};

const EXECUTABLE: &str = "max";
const ICON: &str = "crates/max-app/assets/icon.png";
const ICON_POINT_SIZES: [u32; 5] = [16, 32, 128, 256, 512];

pub fn pack(profile: &str) -> io::Result<PathBuf> {
    let built = workspace::file("target").join(profile).join(EXECUTABLE);
    let app = workspace::file("target").join(format!("{APP_NAME}.app"));
    let contents = app.join("Contents");
    let (programs, resources) = (contents.join("MacOS"), contents.join("Resources"));
    fs::create_dir_all(&programs)?;
    fs::create_dir_all(&resources)?;
    fs::copy(&built, programs.join(EXECUTABLE)).map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", built.display())))?;
    fs::write(contents.join("Info.plist"), information())?;
    write_icon(&workspace::file(ICON), &resources)?;
    app.canonicalize()
}

fn information() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>{APP_NAME}</string>
    <key>CFBundleDisplayName</key><string>{APP_NAME}</string>
    <key>CFBundleIdentifier</key><string>net.kristoph.marquee</string>
    <key>CFBundleExecutable</key><string>{EXECUTABLE}</string>
    <key>CFBundleIconFile</key><string>icon</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>{version}</string>
    <key>CFBundleVersion</key><string>{version}</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"#,
        version = env!("CARGO_PKG_VERSION")
    )
}

/// The system's own tools scale the picture to each size an icon file holds and assemble it.
fn write_icon(picture: &Path, resources: &Path) -> io::Result<()> {
    let sizes = resources.join("icon.iconset");
    fs::create_dir_all(&sizes)?;
    for points in ICON_POINT_SIZES {
        for (scale, suffix) in [(1, ""), (2, "@2x")] {
            let scaled = sizes.join(format!("icon_{points}x{points}{suffix}.png"));
            run(Command::new("sips")
                .arg("-z")
                .args([(points * scale).to_string(), (points * scale).to_string()])
                .arg(picture)
                .arg("--out")
                .arg(scaled))?;
        }
    }
    run(Command::new("iconutil").args(["--convert", "icns", "--output"]).arg(resources.join("icon.icns")).arg(&sizes))?;
    fs::remove_dir_all(sizes)
}
