use std::sync::Arc;

use block2::RcBlock;
use max_api::Error;

use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{NSObject, NSObjectProtocol},
    AnyThread, DefinedClass, Message,
};
use objc2_av_foundation::{AVContentKeyRequest, AVContentKeyResponse, AVContentKeySession, AVContentKeySessionDelegate};
use objc2_foundation::{NSData, NSError, NSString};

const KEY_ADDRESS_PREFIX: &str = "skd://";
const ERROR_DOMAIN: &str = "max.fairplay";

/// Given a key's id and the request the system's player made for it, returns the service's answer.
pub type FetchKey = dyn Fn(&str, &[u8]) -> Result<Vec<u8>, Error> + Send + Sync;

pub struct KeyDelegateState {
    certificate: Vec<u8>,
    fetch_key: Arc<FetchKey>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and the state is only read.
    #[unsafe(super(NSObject))]
    #[name = "MaxFairPlayKeyDelegate"]
    #[ivars = KeyDelegateState]
    pub struct KeyDelegate;

    unsafe impl NSObjectProtocol for KeyDelegate {}

    unsafe impl AVContentKeySessionDelegate for KeyDelegate {
        #[unsafe(method(contentKeySession:didProvideContentKeyRequest:))]
        fn did_provide_request(&self, _session: &AVContentKeySession, request: &AVContentKeyRequest) {
            self.answer(request);
        }
    }
);

impl KeyDelegate {
    pub fn new(certificate: Vec<u8>, fetch_key: Arc<FetchKey>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(KeyDelegateState { certificate, fetch_key });
        // SAFETY: `init` is NSObject's, which takes no arguments.
        unsafe { msg_send![super(this), init] }
    }

    fn answer(&self, request: &AVContentKeyRequest) {
        let Some(key_id) = key_id_of(request) else { return };
        let Some(key_id_bytes) = bytes_of(&key_id) else { return };
        let state = self.ivars();
        let fetch_key = state.fetch_key.clone();
        let awaited = request.retain();
        let on_key_request = RcBlock::new(move |key_request: *mut NSData, _error: *mut NSError| {
            // SAFETY: the system hands over either a valid key request or null.
            let key = match unsafe { key_request.as_ref() } {
                Some(key_request) => fetch_key(&key_id, &key_request.to_vec()),
                None => Err(Error::Lacks("a key request from the system")),
            };
            // SAFETY: the request is retained until this block is dropped.
            unsafe {
                match key {
                    Ok(key) => {
                        let response = AVContentKeyResponse::contentKeyResponseWithFairPlayStreamingKeyResponseData(&NSData::with_bytes(&key));
                        awaited.processContentKeyResponse(&response);
                    }
                    Err(error) => {
                        eprintln!("the key for the title could not be had: {error}");
                        awaited.processContentKeyResponseError(&NSError::errorWithDomain_code_userInfo(&NSString::from_str(ERROR_DOMAIN), 1, None));
                    }
                }
            }
        });
        // SAFETY: the certificate and identifier are plain data, as the method expects.
        unsafe {
            request.makeStreamingContentKeyRequestDataForApp_contentIdentifier_options_completionHandler(
                &NSData::with_bytes(&state.certificate),
                Some(&NSData::with_bytes(&key_id_bytes)),
                None,
                &on_key_request,
            );
        }
    }
}

fn key_id_of(request: &AVContentKeyRequest) -> Option<String> {
    // SAFETY: called on the key session's queue, which is the only place the request is used.
    let identifier = unsafe { request.identifier() }?;
    let address = identifier.downcast_ref::<NSString>()?.to_string();
    Some(address.strip_prefix(KEY_ADDRESS_PREFIX).unwrap_or(&address).to_string())
}

fn bytes_of(key_id: &str) -> Option<Vec<u8>> {
    let hex: String = key_id.chars().filter(|character| *character != '-').collect();
    (0..hex.len()).step_by(2).map(|at| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_a_key_id_into_its_sixteen_bytes() {
        assert_eq!(
            bytes_of("0101d539-f634-f994-3676-bb93bf899daa").unwrap(),
            [1, 1, 0xd5, 0x39, 0xf6, 0x34, 0xf9, 0x94, 0x36, 0x76, 0xbb, 0x93, 0xbf, 0x89, 0x9d, 0xaa]
        );
        assert_eq!(bytes_of("not-hex"), None);
    }
}
