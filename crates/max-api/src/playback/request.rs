use std::hash::{BuildHasher, Hasher};

use serde_json::{json, Value};

use super::progress::SessionIds;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Purpose {
    HeroPreview,
    Title,
}

/// What the web player sends, pared down to H.264 at 720p.
pub(crate) fn playback_request(device_id: &str, edit_id: &str, purpose: Purpose, session_ids: &SessionIds) -> Value {
    let mut request = json!({
        "appBundle": "hbomax",
        "consumptionType": "streaming",
        "deviceInfo": {
            "deviceId": device_id,
            "browser": {"name": "Safari", "version": "26.0"},
            "make": "desktop", "model": "desktop",
            "os": {"name": "macOS", "version": "10.15.7"},
            "platform": "WEB", "deviceType": "web",
            "player": {
                "sdk": {"name": "Beam Player Desktop", "version": "7.13.0"},
                "mediaEngine": {"name": "GLUON_BROWSER", "version": "11.0.0"},
                "playerView": {"height": 720, "width": 1280}
            }
        },
        "editId": edit_id,
        "capabilities": {
            "manifests": {"formats": {"dash": {}}},
            "codecs": {
                "audio": {"decoders": [{"codec": "aac", "profiles": ["lc", "hev", "hev2"]}]},
                "video": {
                    "decoders": [{
                        "codec": "h264", "profiles": ["high", "main", "baseline"], "maxLevel": "5.2",
                        "levelConstraints": {
                            "width": {"min": 0, "max": 1280}, "height": {"min": 0, "max": 720}, "framerate": {"min": 0, "max": 60}
                        }
                    }],
                    "hdrFormats": []
                }
            }
        },
        "gdpr": false,
        "firstPlay": false,
        "playbackSessionId": session_ids.playback,
        "applicationSessionId": session_ids.application,
        "userPreferences": {"videoQuality": "best", "uiLanguage": "en-US"},
        "features": ["mlp"]
    });
    match purpose {
        Purpose::HeroPreview => request["playbackContext"] = json!("browse"),
        Purpose::Title => {
            request["capabilities"]["contentProtection"] =
                json!({"contentDecryptionModules": [{"drmKeySystem": "fairplay", "maxSecurityLevel": "main"}]});
        }
    }
    request
}

impl SessionIds {
    pub(crate) fn new() -> Self {
        Self { playback: random_uuid(), application: random_uuid() }
    }
}

fn random_uuid() -> String {
    // The standard library seeds every `RandomState` differently.
    let random_half = || std::collections::hash_map::RandomState::new().build_hasher().finish();
    let hex = format!("{:016x}{:016x}", random_half(), random_half());
    format!("{}-{}-4{}-a{}-{}", &hex[0..8], &hex[8..12], &hex[13..16], &hex[17..20], &hex[20..32])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_uuids_are_well_formed_and_differ() {
        let (first, second) = (random_uuid(), random_uuid());
        assert_ne!(first, second);
        assert_eq!(first.split('-').map(str::len).collect::<Vec<_>>(), [8, 4, 4, 4, 12]);
    }

    #[test]
    fn only_a_title_request_names_fairplay() {
        let title = playback_request("device", "edit", Purpose::Title, &SessionIds::new());
        assert_eq!(title["capabilities"]["contentProtection"]["contentDecryptionModules"][0]["drmKeySystem"], "fairplay");
        assert!(title.get("playbackContext").is_none());
        let preview = playback_request("device", "edit", Purpose::HeroPreview, &SessionIds::new());
        assert_eq!(preview["playbackContext"], "browse");
        assert!(preview["capabilities"].get("contentProtection").is_none());
    }
}
