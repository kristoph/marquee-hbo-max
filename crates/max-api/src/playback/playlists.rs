use std::thread;

use super::{
    index::{segments, Segment},
    manifest::{Feature, Rendition},
};
use crate::{net, Error};

pub const MASTER_PLAYLIST: &str = "master.m3u8";
const AUDIO_PLAYLIST: &str = "audio.m3u8";
const AUDIO_GROUP: &str = "audio";

pub struct Playlist {
    pub name: String,
    pub text: String,
}

/// Each rendition's index is fetched at the same time as the others, since that waiting is
/// most of what stands between choosing a title and seeing it.
pub(super) fn write(feature: &Feature) -> Result<Vec<Playlist>, Error> {
    let renditions: Vec<&Rendition> = std::iter::once(&feature.audio).chain(&feature.video).collect();
    let media: Vec<Result<String, Error>> = thread::scope(|scope| {
        let fetches: Vec<_> = renditions.iter().map(|rendition| scope.spawn(|| media_playlist_of(rendition))).collect();
        fetches.into_iter().map(|fetch| fetch.join().expect("an index fetch does not panic")).collect()
    });
    let mut media = media.into_iter();
    let mut playlists = vec![Playlist { name: AUDIO_PLAYLIST.to_string(), text: media.next().ok_or(Error::Lacks("sound"))?? }];
    let mut master = vec![
        "#EXTM3U".to_string(),
        "#EXT-X-VERSION:7".to_string(),
        format!(
            "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"{AUDIO_GROUP}\",NAME=\"{language}\",LANGUAGE=\"{language}\",DEFAULT=YES,AUTOSELECT=YES,URI=\"{AUDIO_PLAYLIST}\"",
            language = feature.audio_language
        ),
    ];
    for (number, (video, text)) in feature.video.iter().zip(media).enumerate() {
        let name = format!("video-{number}.m3u8");
        master.push(variant(video, &feature.audio));
        master.push(name.clone());
        playlists.push(Playlist { name, text: text? });
    }
    playlists.push(Playlist { name: MASTER_PLAYLIST.to_string(), text: master.join("\n") + "\n" });
    Ok(playlists)
}

fn media_playlist_of(rendition: &Rendition) -> Result<String, Error> {
    let index = net::byte_range(&rendition.url, rendition.index)?;
    Ok(media_playlist(rendition, &segments(&index, rendition.index)?))
}

fn variant(video: &Rendition, audio: &Rendition) -> String {
    let resolution = video.resolution.map(|(width, height)| format!(",RESOLUTION={width}x{height}")).unwrap_or_default();
    format!(
        "#EXT-X-STREAM-INF:BANDWIDTH={},CODECS=\"{},{}\"{resolution},AUDIO=\"{AUDIO_GROUP}\"",
        video.bandwidth + audio.bandwidth,
        video.codecs,
        audio.codecs
    )
}

/// The key line only names the key; Apple's player asks the app for it, and the app asks the
/// service.
fn media_playlist(rendition: &Rendition, segments: &[Segment]) -> String {
    let longest = segments.iter().map(|segment| segment.seconds).fold(0.0, f64::max);
    let mut lines = vec![
        "#EXTM3U".to_string(),
        "#EXT-X-VERSION:7".to_string(),
        format!("#EXT-X-TARGETDURATION:{}", longest.ceil() as u64),
        "#EXT-X-PLAYLIST-TYPE:VOD".to_string(),
        "#EXT-X-INDEPENDENT-SEGMENTS".to_string(),
        format!(
            "#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"skd://{}\",KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\"",
            rendition.key_id
        ),
        format!("#EXT-X-MAP:URI=\"{}\",BYTERANGE=\"{}@{}\"", rendition.url, rendition.initialization.len(), rendition.initialization.first),
    ];
    for segment in segments {
        lines.push(format!("#EXTINF:{:.5},", segment.seconds));
        lines.push(format!("#EXT-X-BYTERANGE:{}@{}", segment.bytes, segment.offset));
        lines.push(rendition.url.clone());
    }
    lines.push("#EXT-X-ENDLIST".to_string());
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::manifest::{
        feature,
        tests::{MANIFEST, MANIFEST_URL},
    };

    #[test]
    fn a_media_playlist_names_the_key_the_start_of_the_file_and_each_segment() {
        let feature = feature(MANIFEST, MANIFEST_URL).unwrap();
        let parts = [Segment { offset: 2000, bytes: 1000, seconds: 4.004 }, Segment { offset: 3000, bytes: 500, seconds: 2.002 }];
        let playlist = media_playlist(&feature.video[1], &parts);
        let expected = "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:5\n#EXT-X-PLAYLIST-TYPE:VOD\n#EXT-X-INDEPENDENT-SEGMENTS\n\
            #EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"skd://AAAAAAAA-0000-0000-0000-000000000003\",KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\"\n\
            #EXT-X-MAP:URI=\"https://media.example/title/v/high.mp4\",BYTERANGE=\"812@0\"\n\
            #EXTINF:4.00400,\n#EXT-X-BYTERANGE:1000@2000\nhttps://media.example/title/v/high.mp4\n\
            #EXTINF:2.00200,\n#EXT-X-BYTERANGE:500@3000\nhttps://media.example/title/v/high.mp4\n#EXT-X-ENDLIST\n";
        assert_eq!(playlist, expected);
    }

    #[test]
    fn a_variant_carries_both_codecs_and_the_picture_size() {
        let feature = feature(MANIFEST, MANIFEST_URL).unwrap();
        assert_eq!(
            variant(&feature.video[1], &feature.audio),
            "#EXT-X-STREAM-INF:BANDWIDTH=4064000,CODECS=\"avc1.64001f,mp4a.40.5\",RESOLUTION=1024x540,AUDIO=\"audio\""
        );
    }
}
