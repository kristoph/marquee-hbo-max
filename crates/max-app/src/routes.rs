pub const SEARCH: &str = "/search";
pub const MY_STUFF: &str = "/my-stuff";

const TITLE_PREFIXES: [&str; 6] = ["/show/", "/movie/", "/mini-series/", "/topical/", "/sport/", "/standalone/"];

pub fn starts_playback(route: &str) -> bool {
    route.contains("/watch")
}

pub fn leads_to_a_title(route: &str) -> bool {
    starts_playback(route) || TITLE_PREFIXES.iter().any(|prefix| route.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_titles_and_playback_from_browse_pages() {
        assert!(starts_playback("/video/watch/abc"));
        assert!(leads_to_a_title("/video/watch/abc") && leads_to_a_title("/movie/abc") && leads_to_a_title("/show/abc"));
        assert!(!leads_to_a_title("/series") && !leads_to_a_title("/my-stuff") && !starts_playback("/show/abc"));
    }
}
