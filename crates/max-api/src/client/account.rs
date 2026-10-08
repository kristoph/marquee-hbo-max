use super::{Body, Client};
use crate::Error;

impl Client {
    pub fn change(&self, method: &str, path: &str, value: Option<&str>) -> Result<(), Error> {
        let url = format!("{}{path}", self.session.content_origin());
        self.send(method, &url, value.map_or(Body::Empty, Body::Text))?;
        Ok(())
    }
}
