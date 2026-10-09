# Marquee for HBO Max

An independent, native Mac client for HBO Max, written in Rust. The interface is drawn by the
app itself; video is played either by Apple's own player or by the service's web player in an
embedded web view.

A signed build is at <https://kristoph.net/projects/marquee-hbo-max/>.

## Requirements

| | |
|---|---|
| **macOS** | Developed and tested on macOS 26 on Apple silicon. The app uses AppKit, WebKit and AVFoundation directly, so it builds for macOS only. |
| **Xcode command line tools** | For the linker and the system frameworks: `xcode-select --install`. Packing the app also uses `sips`, `iconutil`, `codesign`, `hdiutil` and `notarytool`, which come with them. |
| **Rust** | Stable, 2021 edition. Built with Rust 1.98; no older version has been tried. |
| **An HBO Max subscription** | The app signs in through HBO Max's own page and shows your own account. |

There are no other system dependencies. The fonts and images the app needs are in the
repository and compiled into the binary.

## Building and running

```sh
cargo run --bin max
```

The first run opens HBO Max's sign-in page. After that the session is kept in
`~/Library/Application Support/Marquee for HBO Max/`, and fetched artwork in
`~/Library/Caches/Marquee for HBO Max/`.

Useful options (`max --help` lists them all):

```sh
cargo run --bin max -- --player native    # play titles with Apple's player instead of the web player
cargo run --bin max -- --route /series    # start on another page
MAX_LOG=debug cargo run --bin max         # off, error, warn, info (the default) or debug
```

Started this way, the Dock and menu bar show the program's file name, `max`. To see the app
under its own name and icon, pack it as a bundle:

```sh
cargo build --workspace
cargo run --bin bundle
open "target/Marquee for HBO Max.app"
```

## Checking your work

```sh
cargo test --workspace
cargo clippy --workspace --all-targets    # the workspace lints at clippy::pedantic and is kept clean
cargo fmt --all
```

`max --help-developer` lists options for exercising the app without a person at it: opening
it in the background at a screen position, saving a screenshot once artwork has loaded, and
running a scripted walkthrough.

## Layout

| Crate | What it holds |
|---|---|
| `crates/max-api` | The session, the service's calls, and the content model read from its answers. No user interface. |
| `crates/max-media` | The image cache, and video through AVFoundation: preview clips and protected titles. |
| `crates/max-app` | The app itself, binary `max`: state, drawing, the two players, sign-in. |
| `crates/max-tools` | Developer tools: `bundle`, `page-dump` and `web-capture`. |

Longer test blocks live in a `tests.rs` file in a folder named after the module they test;
short ones sit at the foot of the file.

## Developer tools

- **`page-dump`** lists what the app would show for a page, using the saved session. For
  example `cargo run --bin page-dump -- --live /movies`. `--help` lists the rest.
- **`web-capture`** opens the service's web app in a web view and saves its responses under
  `capture/`, for studying what the web app asks for. `max --capture` then draws the saved
  home page without the network.
- **`bundle`** packs the app, and can sign it, put it in a disk image and have Apple notarise
  it.

## Making a release

This needs a "Developer ID Application" certificate in your keychain, and notarisation
credentials saved under a name of your choosing:

```sh
xcrun notarytool store-credentials <profile> --apple-id <apple id> --team-id <team id>

cargo build --workspace --release
cargo run --bin bundle -- --release --sign "Developer ID Application: <name> (<team id>)" --dmg --notarize <profile>
```

The result is `target/Marquee for HBO Max.dmg`, signed, notarised and with the ticket
attached. The build is for the architecture of the Mac it is made on.

## What the app does and does not do

- You sign in on HBO Max's own page, in a web view. The app never sees your password.
- Video is delivered and protected by HBO Max as it is in their own apps. For protected
  titles the app hands Apple's player the licence the service issues for your session; it
  decrypts nothing and saves no video.
- The app is not an official client, and its requests are not those of one. HBO Max may
  change or refuse what it relies on at any time.

## About

Marquee for HBO Max is an independent effort by Kristoph Cichocki-Romanov
(<hello@kristoph.net>). It is not affiliated with, endorsed by or sponsored by Skydance
Corporation, Warner Bros. Discovery, Inc. or Home Box Office, Inc.

HBO® and HBO Max® are registered trademarks of Home Box Office, Inc. Skydance™ and Warner
Bros.™ are trademarks of their respective owners.

The app's text is set in Noto Sans and its icons come from Font Awesome, both under the SIL
Open Font License 1.1; see `crates/max-app/assets/fonts`.
