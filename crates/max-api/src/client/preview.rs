use serde_json::Value;

use super::{Body, Client};
use crate::{
    net,
    playback::{playback_request, Purpose, SessionIds},
    Error,
};

const TALLEST_PICTURE_WORTH_DECODING: u32 = 720;

#[derive(Debug, Clone, PartialEq)]
pub struct PreviewFiles {
    pub video: String,
    pub audio: Option<String>,
}

impl Client {
    pub fn preview_files(&self, edit_id: &str) -> Result<PreviewFiles, Error> {
        let device_id = self.session.device_id().ok_or(Error::SignedOut)?;
        let url = format!("{}/any/playback/v1/playbackInfo", self.session.playback_origin());
        let answer: Value = serde_json::from_str(&self.send(
            "POST",
            &url,
            Body::Json(&playback_request(device_id, edit_id, Purpose::HeroPreview, &SessionIds::new()).to_string()),
        )?)?;
        if !answer["drm"].is_null() {
            return Err(Error::Protected);
        }
        let manifest_url = answer["manifest"]["url"].as_str().ok_or(Error::Lacks("a manifest"))?;
        let manifest = net::text(manifest_url)?;
        if manifest.contains("ContentProtection") {
            return Err(Error::Protected);
        }
        files_in_manifest(&manifest, manifest_url, TALLEST_PICTURE_WORTH_DECODING).ok_or(Error::Lacks("H.264 video"))
    }
}

struct Rendition<'a> {
    codec: String,
    height: Option<u32>,
    file: &'a str,
}

/// Reads a DASH manifest of the "on-demand" profile, in which every rendition is one whole file.
fn files_in_manifest(manifest: &str, manifest_url: &str, tallest: u32) -> Option<PreviewFiles> {
    let renditions: Vec<Rendition> = manifest.split("<Representation ").skip(1).filter_map(rendition).collect();
    let folder = manifest_url.split('?').next()?.rsplit_once('/')?.0;
    let address = |file: &str| format!("{folder}/{file}");
    let video = renditions
        .iter()
        .filter(|rendition| rendition.codec.starts_with("avc1"))
        .filter_map(|rendition| Some((rendition.height.filter(|height| *height <= tallest)?, rendition.file)))
        .max_by_key(|(height, _)| *height)?;
    let audio = renditions.iter().find(|rendition| rendition.codec.starts_with("mp4a")).map(|rendition| address(rendition.file));
    Some(PreviewFiles { video: address(video.1), audio })
}

fn rendition(markup: &str) -> Option<Rendition<'_>> {
    let tag = &markup[..markup.find('>')?];
    let file = markup.split("</Representation>").next()?.split("<BaseURL>").nth(1)?.split('<').next()?;
    Some(Rendition { codec: attribute(tag, "codecs")?, height: attribute(tag, "height").and_then(|height| height.parse().ok()), file })
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let start = tag.find(&format!("{name}=\""))? + name.len() + 2;
    Some(tag[start..start + tag[start..].find('"')?].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"<MPD><Period>
      <AdaptationSet contentType="audio"><Representation mimeType="audio/mp4" codecs="mp4a.40.5" id="a0">
        <BaseURL>a/x/a0.mp4</BaseURL></Representation></AdaptationSet>
      <AdaptationSet contentType="image"><Representation mimeType="image/jpeg" height="198" id="i0"></Representation></AdaptationSet>
      <AdaptationSet contentType="video">
        <Representation mimeType="video/mp4" codecs="avc1.64001e" width="640" height="360" id="v2"><BaseURL>v/x/v2.mp4</BaseURL></Representation>
        <Representation mimeType="video/mp4" codecs="avc1.64001f" width="1280" height="720" id="v0"><BaseURL>v/x/v0.mp4</BaseURL></Representation>
        <Representation mimeType="video/mp4" codecs="hvc1.1.6" width="1920" height="1080" id="v9"><BaseURL>v/x/v9.mp4</BaseURL></Representation>
      </AdaptationSet></Period></MPD>"#;
    const MANIFEST_URL: &str = "https://media.example/amer/clip/dash.mpd?token=1";

    #[test]
    fn picks_the_tallest_h264_picture_within_the_limit_and_the_sound() {
        let files = files_in_manifest(MANIFEST, MANIFEST_URL, 720).unwrap();
        assert_eq!(files.video, "https://media.example/amer/clip/v/x/v0.mp4");
        assert_eq!(files.audio.as_deref(), Some("https://media.example/amer/clip/a/x/a0.mp4"));
        assert_eq!(files_in_manifest(MANIFEST, MANIFEST_URL, 480).unwrap().video, "https://media.example/amer/clip/v/x/v2.mp4");
    }

    #[test]
    fn a_manifest_without_video_has_no_files() {
        assert_eq!(files_in_manifest("<MPD></MPD>", MANIFEST_URL, 720), None);
    }
}
