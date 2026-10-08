use ureq::http::Request;

use super::Client;
use crate::{net, session::WEB_ORIGIN, Error};

#[derive(Clone, Copy)]
pub enum Body<'a> {
    Empty,
    Text(&'a str),
    Json(&'a str),
}

impl Client {
    pub(crate) fn get(&self, url: &str) -> Result<String, Error> {
        // The API writes square brackets in its queries, which a strict URL parser rejects.
        let url = url.replace('[', "%5B").replace(']', "%5D");
        let mut request = net::agent().get(&url).header("referer", format!("{WEB_ORIGIN}/"));
        for (name, value) in self.session.replayed_headers() {
            request = request.header(name, value);
        }
        Ok(net::accepted(request.call())?.body_mut().read_to_string()?)
    }

    pub(crate) fn send(&self, method: &str, url: &str, body: Body) -> Result<String, Error> {
        let mut request = Request::builder().method(method).uri(url).header("referer", format!("{WEB_ORIGIN}/"));
        for (name, value) in self.session.replayed_headers() {
            request = request.header(name, value);
        }
        let content = match body {
            Body::Empty => "",
            Body::Text(text) => {
                request = request.header("content-type", "text/plain");
                text
            }
            Body::Json(json) => {
                request = request.header("content-type", "application/json");
                json
            }
        };
        let request = request.body(content.to_string()).map_err(ureq::Error::from)?;
        Ok(net::accepted(net::agent().run(request))?.body_mut().read_to_string()?)
    }
}
