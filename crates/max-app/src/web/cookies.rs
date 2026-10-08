use std::ptr::NonNull;

use block2::RcBlock;
use max_api::session::WebCookie;
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSArray, NSDate, NSHTTPCookie, NSHTTPCookieDomain, NSHTTPCookieName, NSHTTPCookiePath, NSHTTPCookiePropertyKey,
    NSHTTPCookieSecure, NSHTTPCookieValue, NSMutableDictionary, NSString,
};
use objc2_web_kit::WKWebsiteDataStore;
use wry::{WebView, WebViewExtMacOS};

use crate::Failure;

// These talk to the system cookie store directly: wry's own cookie calls wait for completion
// by spinning the run loop, which the window toolkit forbids from inside its event handler.

pub fn store(view: &WebView, cookie: &WebCookie) -> Result<(), Failure> {
    let flag = |on: bool| if on { ns_string!("TRUE") } else { ns_string!("FALSE") };
    let (name, value, path) = (NSString::from_str(&cookie.name), NSString::from_str(&cookie.value), NSString::from_str(&cookie.path));
    let domain_and_subdomains = NSString::from_str(&format!(".{}", cookie.domain.trim_start_matches('.')));
    // SAFETY: every property is an NSString, which is what cookie properties take, and this
    // runs on the main thread that owns the web view.
    unsafe {
        let properties: Retained<NSMutableDictionary<NSHTTPCookiePropertyKey, AnyObject>> = NSMutableDictionary::from_slices(
            &[NSHTTPCookieName, NSHTTPCookieValue, NSHTTPCookiePath, NSHTTPCookieDomain, NSHTTPCookieSecure, ns_string!("HttpOnly")],
            &[&*name, &*value, &*path, &*domain_and_subdomains, flag(cookie.secure), flag(cookie.http_only)],
        );
        let native = NSHTTPCookie::cookieWithProperties(&properties).ok_or_else(|| format!("cookie {} was rejected", cookie.name))?;
        view.webview().configuration().websiteDataStore().httpCookieStore().setCookie_completionHandler(&native, None);
    }
    Ok(())
}

pub fn read_all(view: &WebView, deliver: impl Fn(Vec<WebCookie>) + 'static) {
    let completion = RcBlock::new(move |cookies: NonNull<NSArray<NSHTTPCookie>>| {
        // SAFETY: the store hands over a valid array for the duration of the call.
        let cookies = unsafe { cookies.as_ref() }.to_vec();
        deliver(cookies.iter().map(|cookie| web_cookie(cookie)).collect());
    });
    // SAFETY: on the main thread that owns the web view.
    unsafe { view.webview().configuration().websiteDataStore().httpCookieStore().getAllCookies(&completion) }
}

pub fn clear_all(done: impl Fn() + 'static) -> Result<(), Failure> {
    let main_thread = MainThreadMarker::new().ok_or("web data can only be cleared from the main thread")?;
    let completion = RcBlock::new(done);
    // SAFETY: on the main thread.
    unsafe {
        let everything = WKWebsiteDataStore::allWebsiteDataTypes(main_thread);
        WKWebsiteDataStore::defaultDataStore(main_thread).removeDataOfTypes_modifiedSince_completionHandler(
            &everything,
            &NSDate::distantPast(),
            &completion,
        );
    }
    Ok(())
}

fn web_cookie(cookie: &NSHTTPCookie) -> WebCookie {
    WebCookie {
        name: cookie.name().to_string(),
        value: cookie.value().to_string(),
        domain: cookie.domain().to_string(),
        path: cookie.path().to_string(),
        secure: cookie.isSecure(),
        http_only: cookie.isHTTPOnly(),
    }
}
