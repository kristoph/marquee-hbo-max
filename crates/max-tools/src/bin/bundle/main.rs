//! Packs the built app as `Marquee for HBO Max.app`, which is what gives it its name and icon
//! in the Dock: a program started on its own is shown there by its file name. For handing
//! the app to others it can also sign it, put it in a disk image and have Apple notarise it.

mod app;
mod release;

use std::{
    io,
    process::{Command, ExitCode},
};

const APP_NAME: &str = "Marquee for HBO Max";

const USAGE: &str = "\
bundle [options]         pack target/debug/max as an app in target/

  --release              pack target/release/max instead
  --sign <identity>      sign the app as that Developer ID, with the hardened runtime
  --dmg                  put the app in a disk image beside it
  --notarize <profile>   send the disk image to Apple under the credentials saved in the
                         keychain by that name (xcrun notarytool store-credentials), and
                         attach the answer to it
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|argument| argument == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match bundle(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bundle: {error}");
            ExitCode::FAILURE
        }
    }
}

fn bundle(arguments: &[String]) -> io::Result<()> {
    let has = |flag: &str| arguments.iter().any(|argument| argument == flag);
    let value_of = |flag: &str| arguments.iter().position(|argument| argument == flag).and_then(|index| arguments.get(index + 1));
    let identity = value_of("--sign");

    let app = app::pack(if has("--release") { "release" } else { "debug" })?;
    println!("{}", app.display());
    if let Some(identity) = identity {
        release::sign(&app, identity)?;
    }
    if has("--dmg") || has("--notarize") {
        let image = release::disk_image(&app, identity.map(String::as_str))?;
        println!("{}", image.display());
        if let Some(profile) = value_of("--notarize") {
            release::notarize(&image, profile)?;
        }
    }
    Ok(())
}

/// Runs one of the system's own tools, giving back what it printed.
fn run(command: &mut Command) -> io::Result<String> {
    let output = command.output()?;
    match output.status.success() {
        true => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
        false => Err(io::Error::other(format!(
            "{} failed: {}{}",
            command.get_program().display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}
