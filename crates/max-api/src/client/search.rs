use super::Client;
use crate::{cms::Row, Error};

pub const SEARCH_RESULTS_ALIAS: &str = "search-page-rail-results";

impl Client {
    pub fn search(&self, results_id: &str, text: &str, topic_parameters: Option<&str>) -> Result<Row, Error> {
        let typed = (!text.is_empty()).then(|| format!("contentFilter[query]={}", percent_encode(text)));
        let parameters: Vec<&str> = typed.as_deref().into_iter().chain(topic_parameters).collect();
        self.row(results_id, Some(&parameters.join("&")))
    }
}

fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_search_text() {
        assert_eq!(percent_encode("the wire"), "the%20wire");
        assert_eq!(percent_encode("a&b=c#d"), "a%26b%3Dc%23d");
        assert_eq!(percent_encode("señor"), "se%C3%B1or");
    }
}
