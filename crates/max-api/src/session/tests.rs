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
