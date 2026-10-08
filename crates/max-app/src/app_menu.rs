//! The menu the system puts in the menu bar is named after the program file, `max`, and its
//! About panel knows nothing of the app. Both are put right here.

use std::{cell::RefCell, ptr};

use objc2::{
    class, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyObject, NSObject, NSObjectProtocol, Sel},
    sel, AnyThread,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};

use crate::view::about::PARAGRAPHS;

const APP_NAME: &str = "HBO Max";
const ICON_SIZE: f64 = 72.0;
const TEXT_WIDTH: f64 = 400.0;
const MARGIN: f64 = 28.0;
const SPACING: f64 = 12.0;

// AppKit's own numbers for these choices.
const VERTICAL: isize = 1;
const CENTERED_ACROSS: isize = 9;
const CENTERED_TEXT: isize = if cfg!(target_arch = "aarch64") { 1 } else { 2 };
const TITLED_AND_CLOSABLE: usize = 1 | 2;
const BUFFERED: usize = 2;

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and the class holds no state.
    #[unsafe(super(NSObject))]
    #[name = "MaxAboutPanel"]
    struct AboutPanel;

    unsafe impl NSObjectProtocol for AboutPanel {}

    impl AboutPanel {
        #[unsafe(method(showAbout:))]
        fn show_about(&self, _sender: *mut AnyObject) {
            show_about_panel();
        }
    }
);

/// Renames the app menu's own items and has its About item open a panel with the app's text.
pub fn adopt() {
    // SAFETY: called on the main thread once the app has launched, and every menu object is
    // checked for null before it is messaged. A menu item does not retain its target, so the
    // target is deliberately never released.
    unsafe {
        let application: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let main_menu: *mut AnyObject = msg_send![application, mainMenu];
        if main_menu.is_null() {
            return;
        }
        let app_item: *mut AnyObject = msg_send![main_menu, itemAtIndex: 0_isize];
        let app_menu: *mut AnyObject = msg_send![app_item, submenu];
        if app_menu.is_null() {
            return;
        }
        let process: Retained<AnyObject> = msg_send![class!(NSProcessInfo), processInfo];
        let program: Retained<NSString> = msg_send![&*process, processName];
        let target: Retained<AboutPanel> = msg_send![AboutPanel::alloc(), init];
        let target = Retained::into_raw(target);

        let count: isize = msg_send![app_menu, numberOfItems];
        for index in 0..count {
            let item: *mut AnyObject = msg_send![app_menu, itemAtIndex: index];
            let title: Retained<NSString> = msg_send![item, title];
            let renamed = NSString::from_str(&renamed(&title.to_string(), &program.to_string()));
            let _: () = msg_send![item, setTitle: &*renamed];
            let action: Option<Sel> = msg_send![item, action];
            if action == Some(sel!(orderFrontStandardAboutPanel:)) {
                let _: () = msg_send![item, setTarget: target];
                let _: () = msg_send![item, setAction: sel!(showAbout:)];
            }
        }
    }
}

/// "About max" becomes "About HBO Max"; items that do not end in the program's name are kept.
fn renamed(title: &str, program: &str) -> String {
    match title.strip_suffix(program) {
        Some(action) if action.ends_with(' ') => format!("{action}{APP_NAME}"),
        _ => title.to_string(),
    }
}

thread_local! {
    static ABOUT_WINDOW: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

/// The window is built the first time it is asked for and only shown again after that.
fn show_about_panel() {
    ABOUT_WINDOW.with_borrow_mut(|window| {
        let window = window.get_or_insert_with(about_window);
        // SAFETY: the window was made by `about_window` and is kept alive by this thread.
        unsafe {
            let _: () = msg_send![&**window, center];
            let _: () = msg_send![&**window, makeKeyAndOrderFront: ptr::null::<AnyObject>()];
        }
    });
}

/// Laid out as the system's own message boxes are: the icon, the name and the text beneath
/// one another, centred, on the plain window background.
fn about_window() -> Retained<AnyObject> {
    // SAFETY: called on the main thread; every message is one its receiver's class documents,
    // with arguments of the kinds it takes.
    unsafe {
        let application: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        let icon: Retained<AnyObject> = msg_send![application, applicationIconImage];
        let icon: Retained<AnyObject> = msg_send![&*icon, copy];
        let _: () = msg_send![&*icon, setSize: NSSize::new(ICON_SIZE, ICON_SIZE)];
        let icon: Retained<AnyObject> = msg_send![class!(NSImageView), imageViewWithImage: &*icon];

        let body_size: f64 = msg_send![class!(NSFont), systemFontSize];
        let small_size: f64 = msg_send![class!(NSFont), smallSystemFontSize];
        let bold: Retained<AnyObject> = msg_send![class!(NSFont), boldSystemFontOfSize: body_size];
        let regular: Retained<AnyObject> = msg_send![class!(NSFont), systemFontOfSize: body_size];
        let small: Retained<AnyObject> = msg_send![class!(NSFont), systemFontOfSize: small_size];
        let dim: Retained<AnyObject> = msg_send![class!(NSColor), secondaryLabelColor];

        let name = label(APP_NAME, &bold);
        let version = label(concat!("Version ", env!("CARGO_PKG_VERSION")), &small);
        let _: () = msg_send![&*version, setTextColor: &*dim];
        let mut views = vec![icon, name, version];
        views.extend(PARAGRAPHS.iter().map(|paragraph| label(paragraph, &regular)));

        let stack: Retained<AnyObject> = msg_send![class!(NSStackView), stackViewWithViews: &*NSArray::from_retained_slice(&views)];
        let _: () = msg_send![&*stack, setOrientation: VERTICAL];
        let _: () = msg_send![&*stack, setAlignment: CENTERED_ACROSS];
        let _: () = msg_send![&*stack, setSpacing: SPACING];
        let _: () = msg_send![&*stack, setTranslatesAutoresizingMaskIntoConstraints: false];
        let fitting: NSSize = msg_send![&*stack, fittingSize];

        let window: Allocated<AnyObject> = msg_send![class!(NSWindow), alloc];
        let content = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(TEXT_WIDTH + MARGIN * 2.0, fitting.height + MARGIN * 2.0));
        let window: Retained<AnyObject> =
            msg_send![window, initWithContentRect: content, styleMask: TITLED_AND_CLOSABLE, backing: BUFFERED, defer: false];
        let _: () = msg_send![&*window, setReleasedWhenClosed: false];
        let content: Retained<AnyObject> = msg_send![&*window, contentView];
        let _: () = msg_send![&*content, addSubview: &*stack];
        for anchor in [sel!(centerXAnchor), sel!(centerYAnchor)] {
            let of_stack: Retained<AnyObject> = msg_send![&*stack, performSelector: anchor];
            let of_content: Retained<AnyObject> = msg_send![&*content, performSelector: anchor];
            let centred: Retained<AnyObject> = msg_send![&*of_stack, constraintEqualToAnchor: &*of_content];
            let _: () = msg_send![&*centred, setActive: true];
        }
        window
    }
}

/// # Safety
/// Must be called on the main thread, with `font` an `NSFont`.
unsafe fn label(text: &str, font: &AnyObject) -> Retained<AnyObject> {
    let label: Retained<AnyObject> = msg_send![class!(NSTextField), wrappingLabelWithString: &*NSString::from_str(text)];
    let _: () = msg_send![&*label, setFont: font];
    let _: () = msg_send![&*label, setAlignment: CENTERED_TEXT];
    let _: () = msg_send![&*label, setSelectable: false];
    let _: () = msg_send![&*label, setPreferredMaxLayoutWidth: TEXT_WIDTH];
    label
}

#[cfg(test)]
mod tests {
    use super::renamed;

    #[test]
    fn items_named_after_the_program_take_the_apps_name() {
        assert_eq!(renamed("About max", "max"), "About HBO Max");
        assert_eq!(renamed("Quit max", "max"), "Quit HBO Max");
    }

    #[test]
    fn other_items_keep_their_titles() {
        assert_eq!(renamed("Hide Others", "max"), "Hide Others");
        assert_eq!(renamed("Climax", "max"), "Climax");
        assert_eq!(renamed("", "max"), "");
    }
}
