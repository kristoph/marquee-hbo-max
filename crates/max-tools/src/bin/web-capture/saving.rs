use std::{fs, io::Write, os::unix::fs::OpenOptionsExt};

use max_api::{
    session::{Session, WebCookie},
    storage, workspace,
};
use serde_json::Value;
use wry::WebView;

use crate::Failure;

pub const HOME_CAPTURE: &str = "routes-home";
const OWNER_READ_WRITE: u32 = 0o600;

/// Captures hold session tokens and account data, so only their owner may read them.
pub fn save_capture(message: &str) -> Result<String, Failure> {
    let mut capture: Value = serde_json::from_str(message)?;
    let name = capture["capture"].as_str().ok_or("missing capture name")?.to_string();
    if let Some(body) = capture["body"].as_str().and_then(|body| serde_json::from_str::<Value>(body).ok()) {
        capture["body"] = body;
    }
    let path = workspace::capture(&name);
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(OWNER_READ_WRITE).open(path)?;
    file.write_all(serde_json::to_string_pretty(&capture)?.as_bytes())?;
    Ok(name)
}

pub fn save_session(web_view: &WebView) -> Result<(), Failure> {
    let capture: Value = serde_json::from_str(&fs::read_to_string(workspace::capture(HOME_CAPTURE))?)?;
    let home_url = capture["href"].as_str().ok_or("capture has no href")?.to_string();
    let headers = serde_json::from_value(capture["requestHeaders"].clone())?;
    let cookies = web_view
        .cookies()?
        .iter()
        .map(|cookie| WebCookie {
            name: cookie.name().to_string(),
            value: cookie.value().to_string(),
            domain: cookie.domain().unwrap_or_default().to_string(),
            path: cookie.path().unwrap_or("/").to_string(),
            secure: cookie.secure().unwrap_or(false),
            http_only: cookie.http_only().unwrap_or(false),
        })
        .collect();
    Ok(Session::from_web_player(home_url, headers, cookies)?.save(storage::session())?)
}
