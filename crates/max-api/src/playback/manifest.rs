use crate::{net::ByteRange, Error};

const PREFERRED_AUDIO_LANGUAGE: &str = "en";

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Rendition {
    pub url: String,
    pub codecs: String,
    pub bandwidth: u64,
    pub resolution: Option<(u32, u32)>,
    pub initialization: ByteRange,
    pub index: ByteRange,
    pub key_id: String,
}

#[derive(Debug, PartialEq)]
pub(super) struct Feature {
    pub video: Vec<Rendition>,
    pub audio: Rendition,
    pub audio_language: String,
}

/// A title's manifest has a short unprotected opening and then the title itself, as periods;
/// the title is the protected one, each of whose renditions is one indexed file.
pub(super) fn feature(manifest: &str, manifest_url: &str) -> Result<Feature, Error> {
    let folder = manifest_url
        .split('?')
        .next()
        .and_then(|url| url.rsplit_once('/'))
        .map(|(folder, _)| folder)
        .ok_or(Error::Lacks("an address for the manifest's files"))?;
    let period = manifest
        .split("<Period")
        .skip(1)
        .filter(|period| period.contains("ContentProtection"))
        .last()
        .ok_or(Error::Lacks("a protected period in the manifest"))?;
    let sets: Vec<&str> = period.split("<AdaptationSet").skip(1).collect();
    let renditions_of = |set: &str| -> Vec<Rendition> {
        let key_id = attribute(set, "cenc:default_KID").unwrap_or_default();
        set.split("<Representation").skip(1).filter_map(|markup| rendition(markup, folder, &key_id)).collect()
    };
    let of_kind = |kind: &'static str| sets.iter().copied().filter(move |set| attribute(tag(set), "contentType").as_deref() == Some(kind));

    let video: Vec<Rendition> = of_kind("video")
        .map(renditions_of)
        .find(|renditions| renditions.iter().any(|rendition| rendition.codecs.starts_with("avc1")))
        .unwrap_or_default();
    let language = |set: &str| attribute(tag(set), "lang").unwrap_or_default();
    let audio_sets: Vec<&str> = of_kind("audio").collect();
    let preferred = audio_sets.iter().filter(|set| language(set).starts_with(PREFERRED_AUDIO_LANGUAGE));
    let audio_set = preferred.clone().find(|set| set.contains("codecs=\"mp4a")).or_else(|| preferred.clone().next()).or(audio_sets.first());
    let audio = audio_set.and_then(|set| Some((renditions_of(set).into_iter().next()?, language(set))));
    match (video.is_empty(), audio) {
        (false, Some((audio, audio_language))) => Ok(Feature { video, audio, audio_language }),
        _ => Err(Error::Lacks("H.264 video with sound")),
    }
}

fn rendition(markup: &str, folder: &str, key_id: &str) -> Option<Rendition> {
    let attributes = tag(markup);
    let file = markup.split("</Representation>").next()?.split("<BaseURL>").nth(1)?.split('<').next()?;
    let number = |name: &str| attribute(attributes, name)?.parse::<u32>().ok();
    Some(Rendition {
        url: if file.contains("://") { file.to_string() } else { format!("{folder}/{file}") },
        codecs: attribute(attributes, "codecs")?,
        bandwidth: attribute(attributes, "bandwidth")?.parse().ok()?,
        resolution: number("width").zip(number("height")),
        initialization: byte_range(&attribute(markup, "range")?)?,
        index: byte_range(&attribute(markup, "indexRange")?)?,
        key_id: key_id.to_string(),
    })
}

fn tag(markup: &str) -> &str {
    &markup[..markup.find('>').unwrap_or(markup.len())]
}

fn attribute(markup: &str, name: &str) -> Option<String> {
    let start = markup.find(&format!(" {name}=\""))? + name.len() + 3;
    Some(markup[start..start + markup[start..].find('"')?].to_string())
}

fn byte_range(text: &str) -> Option<ByteRange> {
    let (first, last) = text.split_once('-')?;
    Some(ByteRange { first: first.parse().ok()?, last: last.parse().ok()? })
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub const MANIFEST: &str = r#"<MPD>
      <Period id="0"><BaseURL>https://free.example/x/</BaseURL>
        <AdaptationSet contentType="video"><Representation codecs="avc1.64001f" bandwidth="1" width="1280" height="720" id="o"></Representation></AdaptationSet>
      </Period>
      <Period id="1">
        <AdaptationSet id="0" lang="fr-FR" contentType="audio">
          <ContentProtection schemeIdUri="urn:mpeg:dash:mp4protection:2011" cenc:default_KID="AAAAAAAA-0000-0000-0000-000000000001" value="cbcs"></ContentProtection>
          <Representation mimeType="audio/mp4" bandwidth="64000" codecs="mp4a.40.5" id="a0"><BaseURL>a/fr.mp4</BaseURL>
            <SegmentBase timescale="48000" indexRange="700-999"><Initialization range="0-699"></Initialization></SegmentBase></Representation>
        </AdaptationSet>
        <AdaptationSet id="1" lang="en-US" contentType="audio">
          <ContentProtection schemeIdUri="urn:mpeg:dash:mp4protection:2011" cenc:default_KID="AAAAAAAA-0000-0000-0000-000000000002" value="cbcs"></ContentProtection>
          <Representation mimeType="audio/mp4" bandwidth="128000" codecs="ec-3" id="a1"><BaseURL>a/en-surround.mp4</BaseURL>
            <SegmentBase timescale="48000" indexRange="700-999"><Initialization range="0-699"></Initialization></SegmentBase></Representation>
        </AdaptationSet>
        <AdaptationSet id="2" lang="en-US" contentType="audio">
          <ContentProtection schemeIdUri="urn:mpeg:dash:mp4protection:2011" cenc:default_KID="AAAAAAAA-0000-0000-0000-000000000002" value="cbcs"></ContentProtection>
          <Representation mimeType="audio/mp4" bandwidth="64000" codecs="mp4a.40.5" id="a2"><BaseURL>a/en.mp4</BaseURL>
            <SegmentBase timescale="48000" indexRange="675-899"><Initialization range="0-674"></Initialization></SegmentBase></Representation>
        </AdaptationSet>
        <AdaptationSet id="3" maxWidth="1024" contentType="video">
          <ContentProtection schemeIdUri="urn:mpeg:dash:mp4protection:2011" cenc:default_KID="AAAAAAAA-0000-0000-0000-000000000003" value="cbcs"></ContentProtection>
          <Representation mimeType="video/mp4" bandwidth="500000" codecs="avc1.640015" width="508" height="286" id="v2"><BaseURL>v/low.mp4</BaseURL>
            <SegmentBase timescale="24000" indexRange="812-1999"><Initialization range="0-811"></Initialization></SegmentBase></Representation>
          <Representation mimeType="video/mp4" bandwidth="4000000" codecs="avc1.64001f" width="1024" height="540" id="v4"><BaseURL>v/high.mp4</BaseURL>
            <SegmentBase timescale="24000" indexRange="812-1999"><Initialization range="0-811"></Initialization></SegmentBase></Representation>
        </AdaptationSet>
      </Period></MPD>"#;
    pub const MANIFEST_URL: &str = "https://media.example/title/dash.mpd?token=1";

    #[test]
    fn reads_the_protected_periods_video_and_english_stereo_sound() {
        let feature = feature(MANIFEST, MANIFEST_URL).unwrap();
        assert_eq!(
            feature.video.iter().map(|rendition| (rendition.resolution, rendition.bandwidth)).collect::<Vec<_>>(),
            [(Some((508, 286)), 500_000), (Some((1024, 540)), 4_000_000)]
        );
        let high = &feature.video[1];
        assert_eq!(high.url, "https://media.example/title/v/high.mp4");
        assert_eq!((high.initialization, high.index), (ByteRange { first: 0, last: 811 }, ByteRange { first: 812, last: 1999 }));
        assert_eq!(high.key_id, "AAAAAAAA-0000-0000-0000-000000000003");
        assert_eq!((feature.audio.url.as_str(), feature.audio_language.as_str()), ("https://media.example/title/a/en.mp4", "en-US"));
        assert_eq!(feature.audio.key_id, "AAAAAAAA-0000-0000-0000-000000000002");
    }

    #[test]
    fn a_manifest_with_nothing_protected_has_no_feature() {
        assert!(feature("<MPD><Period></Period></MPD>", MANIFEST_URL).is_err());
    }
}
