use std::{collections::BTreeMap, fs, io::Write, os::unix::fs::OpenOptionsExt, path::Path};

use serde::{Deserialize, Serialize};

use crate::Error;

pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
                              (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
pub const WEB_ORIGIN: &str = "https://play.hbomax.com";
pub const SIGN_IN_URL: &str = "https://auth.hbomax.com/login?flow=login";
pub const SESSION_COOKIE_NAME: &str = "st";
pub const SERVICE_DOMAIN: &str = "hbomax.com";

const HOME_ROUTE_PATH: &str = "/cms/routes/home";
const HOME_QUERY: &str = "include=default&decorators=viewingHistory,isFavorite,contentAction,badges&page[items.size]=10";
const ANY_REGION: &str = "any";
const HEADERS_NOT_REPLAYED: [&str; 3] = ["content-type", "traceparent", "tracestate"];
const OWNER_READ_WRITE: u32 = 0o600;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub cookie: String,
    #[serde(default)]
    pub web_cookies: Vec<WebCookie>,
    pub headers: BTreeMap<String, String>,
    pub home_url: String,
}

impl Session {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        if let Some(folder) = path.as_ref().parent() {
            fs::create_dir_all(folder)?;
        }
        let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).mode(OWNER_READ_WRITE).open(path)?;
        file.write_all(serde_json::to_string_pretty(self)?.as_bytes())?;
        Ok(())
    }

    /// The service's hosts are named `default.<kind>-<region>.…`; content comes from the
    /// `any` kind of the account's own region, which a host serving every region does not name.
    pub fn home_url_in_region_of(api_origin: &str) -> Option<String> {
        let (scheme_and_realm, rest) = api_origin.split_once('.')?;
        let (kind_and_region, domain) = rest.split_once('.')?;
        let region = kind_and_region.split_once('-')?.1;
        (region != ANY_REGION).then(|| format!("{scheme_and_realm}.any-{region}.{domain}{HOME_ROUTE_PATH}?{HOME_QUERY}"))
    }

    pub fn from_web_player(home_url: String, headers: BTreeMap<String, String>, cookies: Vec<WebCookie>) -> Result<Self, Error> {
        let token = cookies.iter().find(|cookie| cookie.name == SESSION_COOKIE_NAME).ok_or(Error::SignedOut)?;
        let cookie = format!("{}={}", token.name, token.value);
        let web_cookies = cookies.into_iter().filter(|cookie| cookie.domain.ends_with(SERVICE_DOMAIN)).collect();
        Ok(Self { cookie, web_cookies, headers, home_url })
    }

    pub fn content_origin(&self) -> &str {
        let after_scheme = self.home_url.find("://").map_or(0, |index| index + 3);
        let end = self.home_url[after_scheme..].find('/').map_or(self.home_url.len(), |index| after_scheme + index);
        &self.home_url[..end]
    }

    pub fn account_origin(&self) -> String {
        self.content_origin().replacen(".any-", ".beam-", 1)
    }

    pub fn progress_origin(&self) -> String {
        self.content_origin().replacen("://default.", "://busy.", 1)
    }

    pub fn playback_origin(&self) -> String {
        self.content_origin().replacen(".any-amer.", ".any-any.", 1)
    }

    pub fn page_url(&self, route: &str) -> String {
        self.home_url.replacen(HOME_ROUTE_PATH, &format!("/cms/routes{route}"), 1)
    }

    /// The header reads `<app>/<version> (<make>/<model>; <system>/<version>; <device id>/<client id>)`.
    pub fn device_id(&self) -> Option<&str> {
        let identifiers = self.headers.get("x-device-info")?.rsplit("; ").next()?;
        identifiers.split('/').next().filter(|id| !id.is_empty())
    }

    pub(crate) fn replayed_headers(&self) -> impl Iterator<Item = (&str, &str)> {
        let fixed = [("origin", WEB_ORIGIN), ("cookie", self.cookie.as_str())];
        let captured = self
            .headers
            .iter()
            .filter(|(name, _)| !HEADERS_NOT_REPLAYED.contains(&name.as_str()))
            .map(|(name, value)| (name.as_str(), value.as_str()));
        fixed.into_iter().chain(captured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(home_url: &str, device_info: &str) -> Session {
        let headers = BTreeMap::from([("x-device-info".to_string(), device_info.to_string())]);
        Session { cookie: String::new(), web_cookies: Vec::new(), headers, home_url: home_url.to_string() }
    }

    #[test]
    fn keeps_the_token_for_requests_and_the_services_cookies_for_the_player() {
        let cookie = |name: &str, domain: &str| WebCookie {
            name: name.to_string(),
            value: "v".to_string(),
            domain: domain.to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: true,
        };
        let cookies = vec![cookie("st", "api.hbomax.com"), cookie("consent", ".hbomax.com"), cookie("tracker", "ads.example.com")];
        let session = Session::from_web_player("https://x/cms/routes/home".to_string(), BTreeMap::new(), cookies).unwrap();
        assert_eq!(session.cookie, "st=v");
        assert_eq!(session.web_cookies.iter().map(|cookie| cookie.name.as_str()).collect::<Vec<_>>(), ["st", "consent"]);
        assert!(Session::from_web_player(String::new(), BTreeMap::new(), vec![cookie("consent", ".hbomax.com")]).is_err());
    }

    #[test]
    fn builds_the_home_url_for_the_region_an_api_host_names() {
        assert_eq!(
            Session::home_url_in_region_of("https://default.beam-amer.prd.api.example.com").as_deref(),
            Some("https://default.any-amer.prd.api.example.com/cms/routes/home?include=default&decorators=viewingHistory,isFavorite,contentAction,badges&page[items.size]=10")
        );
        assert_eq!(Session::home_url_in_region_of("https://default.any-any.prd.api.example.com"), None);
    }

    #[test]
    fn reads_the_device_id_from_its_header() {
        let session = session("", "app/7.13.0 (desktop/desktop; macOS/10.15.7; dev-1/client-2)");
        assert_eq!(session.device_id(), Some("dev-1"));
    }

    #[test]
    fn derives_each_origin_and_page_url_from_the_home_url() {
        let session = session("https://default.any-amer.prd.api.example.com/cms/routes/home?include=default", "");
        assert_eq!(session.content_origin(), "https://default.any-amer.prd.api.example.com");
        assert_eq!(session.account_origin(), "https://default.beam-amer.prd.api.example.com");
        assert_eq!(session.playback_origin(), "https://default.any-any.prd.api.example.com");
        assert_eq!(session.progress_origin(), "https://busy.any-amer.prd.api.example.com");
        assert_eq!(session.page_url("/series"), "https://default.any-amer.prd.api.example.com/cms/routes/series?include=default");
    }
}
