use std::io;

use serde_json::Value;

const SESSION_NOT_VALID: &str = "invalid.token";
const UNAUTHORIZED: u16 = 401;
const NOT_FOUND: u16 = 404;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the session is missing or no longer valid")]
    SignedOut,
    #[error("the service has nothing at that address")]
    NotFound,
    #[error("the service refused with status {status} ({code}): {detail}")]
    Refused { status: u16, code: String, detail: String },
    #[error("the service could not be reached: {0}")]
    Unreachable(#[source] Box<ureq::Error>),
    #[error("the service's answer could not be read: {0}")]
    Unreadable(#[from] serde_json::Error),
    #[error("the service's answer lacks {0}")]
    Lacks(&'static str),
    #[error("this is protected and cannot be played here")]
    Protected,
    #[error("the key the service sent is not base64: {0}")]
    KeyEncoding(#[from] base64::DecodeError),
    #[error(transparent)]
    File(#[from] io::Error),
}

impl Error {
    /// Whether signing in again is what would put this right.
    pub fn is_signed_out(&self) -> bool {
        matches!(self, Self::SignedOut)
    }

    /// The service explains a refusal as `{"errors": [{"code": …, "message" or "detail": …}]}`.
    pub(crate) fn refusal(status: u16, answer: &str) -> Self {
        let explanation = serde_json::from_str::<Value>(answer).ok().map(|answer| answer["errors"][0].clone()).unwrap_or_default();
        let text = |name: &str| explanation[name].as_str().map(str::to_string);
        let code = text("code").unwrap_or_default();
        match (status, code.as_str()) {
            (UNAUTHORIZED, _) | (_, SESSION_NOT_VALID) => Self::SignedOut,
            (NOT_FOUND, _) => Self::NotFound,
            _ => Self::Refused { status, code, detail: text("detail").or_else(|| text("message")).unwrap_or_default() },
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Self::Unreachable(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_the_service_no_longer_accepts_means_signed_out() {
        let answer = r#"{"errors":[{"status":"400","code":"invalid.token","message":"Token is missing or not valid"}]}"#;
        assert!(Error::refusal(400, answer).is_signed_out());
        assert!(Error::refusal(401, "").is_signed_out());
    }

    #[test]
    fn a_missing_page_is_told_from_other_refusals() {
        let missing = r#"{"errors": [{"status": "404", "code": "not.found", "detail": "No matching page found"}]}"#;
        assert!(matches!(Error::refusal(404, missing), Error::NotFound));
        let refused = Error::refusal(403, r#"{"errors": [{"code": "access.denied", "detail": "Not in your plan"}]}"#);
        assert_eq!(refused.to_string(), "the service refused with status 403 (access.denied): Not in your plan");
    }

    #[test]
    fn a_refusal_that_explains_nothing_still_carries_its_status() {
        assert!(matches!(Error::refusal(500, "<html>"), Error::Refused { status: 500, .. }));
    }
}
