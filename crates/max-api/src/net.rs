//! One agent for every request, so that connections to a host are kept and reused instead of
//! being set up afresh, with a new handshake, each time.

use std::{io::Read, sync::OnceLock};

use ureq::{http::Response, Agent, Body};

use crate::{session::USER_AGENT, Error};

const LONGEST_REFUSAL_READ: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub first: u64,
    pub last: u64,
}

impl ByteRange {
    pub fn len(self) -> u64 {
        self.last - self.first + 1
    }

    pub fn is_empty(self) -> bool {
        self.last < self.first
    }
}

/// The agent hands back refusals as answers, so that [`accepted`] can read why.
pub(crate) fn agent() -> &'static Agent {
    static AGENT: OnceLock<Agent> = OnceLock::new();
    AGENT.get_or_init(|| Agent::config_builder().user_agent(USER_AGENT).http_status_as_error(false).build().into())
}

pub(crate) fn accepted(answer: Result<Response<Body>, ureq::Error>) -> Result<Response<Body>, Error> {
    let mut answer = answer?;
    if answer.status().is_client_error() || answer.status().is_server_error() {
        let mut explanation = String::new();
        let _ = answer.body_mut().as_reader().take(LONGEST_REFUSAL_READ).read_to_string(&mut explanation);
        return Err(Error::refusal(answer.status().as_u16(), &explanation));
    }
    Ok(answer)
}

pub fn text(url: &str) -> Result<String, Error> {
    Ok(accepted(agent().get(url).call())?.body_mut().read_to_string()?)
}

pub fn bytes(url: &str) -> Result<Vec<u8>, Error> {
    Ok(accepted(agent().get(url).call())?.body_mut().read_to_vec()?)
}

pub fn bytes_up_to(url: &str, most: u64) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    accepted(agent().get(url).call())?.body_mut().as_reader().take(most).read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn byte_range(url: &str, range: ByteRange) -> Result<Vec<u8>, Error> {
    let wanted = format!("bytes={}-{}", range.first, range.last);
    Ok(accepted(agent().get(url).header("range", wanted).call())?.body_mut().read_to_vec()?)
}
