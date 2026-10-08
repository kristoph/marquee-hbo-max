//! Packs the built app as `Marquee for HBO Max.app`, which is what gives it its name and icon
//! in the Dock: a program started on its own is shown there by its file name.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use max_api::workspace;

const APP_NAME: &str = "Marquee for HBO Max";
const EXECUTABLE: &str = "max";
const ICON: &str = "crates/max-app/assets/icon.png";
const ICON_POINT_SIZES: [u32; 5] = [16, 32, 128, 256, 512];

const USAGE: &str = "\
bundle [--release]   pack target/debug/max (or target/release/max) as an app in target/
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|argument| argument == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let profile = if arguments.iter().any(|argument| argument == "--release") { "release" } else { "debug" };
    match pack(profile) {
        Ok(app) => {
            println!("{}", app.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("bundle: {error}");
            ExitCode::FAILURE
        }
    }
}

fn pack(profile: &str) -> io::Result<PathBuf> {
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

fn run(command: &mut Command) -> io::Result<()> {
    let output = command.output()?;
    match output.status.success() {
        true => Ok(()),
        false => Err(io::Error::other(format!("{} failed: {}", command.get_program().display(), String::from_utf8_lossy(&output.stderr).trim()))),
    }
}
