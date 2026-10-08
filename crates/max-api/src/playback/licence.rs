use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

use crate::{client::Client, net, session::WEB_ORIGIN, Error};

impl Client {
    /// Exchanges the key request Apple's player made for the key the service answers with. The
    /// address carries its own authorisation, so nothing of the session is sent.
    pub fn fairplay_key(&self, licence_url: &str, key_id: &str, key_request: &[u8]) -> Result<Vec<u8>, Error> {
        let sent = net::agent()
            .post(licence_url)
            .header("origin", WEB_ORIGIN)
            .header("referer", format!("{WEB_ORIGIN}/"))
            .header("content-type", "application/json")
            .send(licence_request(key_id, key_request).to_string());
        key_in(&net::accepted(sent)?.body_mut().read_to_string()?)
    }
}

fn licence_request(key_id: &str, key_request: &[u8]) -> Value {
    json!({"assetId": asset_id(key_id), "spc": STANDARD.encode(key_request)})
}

pub fn asset_id(key_id: &str) -> String {
    key_id.replace('-', "").to_lowercase()
}

fn key_in(answer: &str) -> Result<Vec<u8>, Error> {
    let answer: Value = serde_json::from_str(answer)?;
    Ok(STANDARD.decode(answer["ckc"].as_str().ok_or(Error::Lacks("a key"))?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_for_a_key_by_its_undashed_id() {
        let request = licence_request("AAAAAAAA-0000-0000-0000-00000000000F", &[1, 2, 3]);
        assert_eq!(request, json!({"assetId": "aaaaaaaa00000000000000000000000f", "spc": "AQID"}));
    }

    #[test]
    fn reads_the_key_out_of_the_answer() {
        assert_eq!(key_in(r#"{"assetId": "x", "ckc": "AQID"}"#).unwrap(), [1, 2, 3]);
        assert!(key_in(r#"{"errors": null}"#).is_err());
    }
}
